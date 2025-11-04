const std = @import("std");
const mem = std.mem;
const process = std.process;

const UsageError = error{InvalidArgs};

const Args = struct {
    opts: ?[]const u8,
    path: ?[]const u8,
};

pub fn main() !void {
    const allocator = std.heap.c_allocator;
    const cmd_args = try process.argsAlloc(allocator);
    defer process.argsFree(allocator, cmd_args);

    const args: Args = switch (cmd_args.len) {
        3 => .{ .opts = cmd_args[1], .path = cmd_args[2] },
        2 => if (mem.startsWith(u8, cmd_args[1], "-"))
            .{ .opts = cmd_args[1], .path = null }
        else
            .{ .opts = null, .path = cmd_args[1] },
        1 => .{ .opts = null, .path = null },
        else => {
            std.debug.print("Usage: ccwc <option> <file>\n", .{});
            return UsageError.InvalidArgs;
        },
    };

    const fd = if (args.path) |path| blk: {
        const f = try std.fs.cwd().openFile(path, .{});
        break :blk f.handle;
    } else std.posix.STDIN_FILENO;
    defer if (args.path != null) std.posix.close(fd);

    const counts = try count_all(fd);

    const result = if (args.opts) |opt| blk: {
        if (mem.eql(u8, opt, "-c")) {
            break :blk try std.fmt.allocPrint(allocator, "{d:>7}", .{counts.bytes});
        } else if (mem.eql(u8, opt, "-l")) {
            break :blk try std.fmt.allocPrint(allocator, "{d:>7}", .{counts.lines});
        } else if (mem.eql(u8, opt, "-w")) {
            break :blk try std.fmt.allocPrint(allocator, "{d:>7}", .{counts.words});
        } else if (mem.eql(u8, opt, "-m")) {
            break :blk try std.fmt.allocPrint(allocator, "{d:>7}", .{counts.chars});
        } else {
            std.debug.print("Unknown option: {s}\nValid options: -c, -l, -w, -m\n", .{opt});
            return UsageError.InvalidArgs;
        }
    } else try std.fmt.allocPrint(allocator, "{d:>7} {d:>7} {d:>7}", .{ counts.lines, counts.words, counts.bytes });
    defer allocator.free(result);

    if (args.path) |path| {
        std.debug.print("{s} {s}\n", .{ result, path });
    } else {
        std.debug.print("{s}\n", .{result});
    }
}

const Counts = struct {
    bytes: u64,
    lines: u64,
    words: u64,
    chars: u64,
};

pub fn count_all(fd: std.posix.fd_t) !Counts {
    const allocator = std.heap.page_allocator;
    var list: std.ArrayList(u8) = .empty;
    defer list.deinit(allocator);

    var chunk_buffer: [4096]u8 = undefined;
    while (true) {
        const n = std.posix.read(fd, &chunk_buffer) catch |err| {
            if (err == error.EndOfStream) break;
            return err;
        };
        if (n == 0) break;
        try list.appendSlice(allocator, chunk_buffer[0..n]);
    }
    const bytes = list.items;

    var counts: Counts = .{ .bytes = @as(u64, bytes.len), .lines = 0, .words = 0, .chars = 0 };
    var in_word: bool = false;

    var i: usize = 0;
    while (i < bytes.len) : (i += 1) {
        const b = bytes[i];
        if (b == 0x0A) counts.lines += 1;

        const is_space = switch (b) {
            0x20, 0x09, 0x0A, 0x0D, 0x0B, 0x0C => true,
            else => false,
        };

        const is_continuation = (b & 0xC0) == 0x80;
        if (!is_space) {
            if (!in_word and !is_continuation) counts.words += 1;
            in_word = true;
        } else {
            in_word = false;
        }

        if (!is_continuation) counts.chars += 1;
    }

    return counts;
}

// Test helper function to create a temporary file and count
fn testCountFromFile(content: []const u8, expected: Counts) !void {
    const allocator = std.testing.allocator;

    const timestamp = std.time.milliTimestamp();
    const filename = try std.fmt.allocPrint(allocator, "ccwc_test_{d}.tmp", .{timestamp});
    defer allocator.free(filename);

    const file = try std.fs.cwd().createFile(filename, .{});
    try file.writeAll(content);
    file.close();
    defer std.fs.cwd().deleteFile(filename) catch {};

    const test_file = try std.fs.cwd().openFile(filename, .{});
    defer test_file.close();

    const counts = try count_all(test_file.handle);
    try std.testing.expectEqual(expected.bytes, counts.bytes);
    try std.testing.expectEqual(expected.lines, counts.lines);
    try std.testing.expectEqual(expected.words, counts.words);
    try std.testing.expectEqual(expected.chars, counts.chars);
}

// Test helper function to count from a pipe (simulated stdin)
fn testCountFromPipe(content: []const u8, expected: Counts) !void {
    const pipe = try std.posix.pipe();
    const read_fd = pipe[0];
    const write_fd = pipe[1];
    defer std.posix.close(read_fd);

    _ = try std.posix.write(write_fd, content);
    std.posix.close(write_fd);

    const counts = try count_all(read_fd);
    try std.testing.expectEqual(expected.bytes, counts.bytes);
    try std.testing.expectEqual(expected.lines, counts.lines);
    try std.testing.expectEqual(expected.words, counts.words);
    try std.testing.expectEqual(expected.chars, counts.chars);
}

test "count from file - basic" {
    try testCountFromFile("hello\n", .{ .bytes = 6, .lines = 1, .words = 1, .chars = 6 });
}

test "count from file - multiple lines" {
    try testCountFromFile("line1\nline2\nline3\n", .{ .bytes = 18, .lines = 3, .words = 3, .chars = 18 });
}

test "count from file - multiple words" {
    try testCountFromFile("one two\tthree\nfour", .{ .bytes = 18, .lines = 1, .words = 4, .chars = 18 });
}

test "count from file - unicode" {
    try testCountFromFile("hélló", .{ .bytes = 7, .lines = 0, .words = 1, .chars = 5 });
}

test "count from file - empty" {
    try testCountFromFile("", .{ .bytes = 0, .lines = 0, .words = 0, .chars = 0 });
}

test "count from pipe - basic" {
    try testCountFromPipe("hello\n", .{ .bytes = 6, .lines = 1, .words = 1, .chars = 6 });
}

test "count from pipe - multiple lines" {
    try testCountFromPipe("line1\nline2\nline3\n", .{ .bytes = 18, .lines = 3, .words = 3, .chars = 18 });
}

test "count from pipe - multiple words" {
    try testCountFromPipe("one two\tthree\nfour", .{ .bytes = 18, .lines = 1, .words = 4, .chars = 18 });
}

test "count from pipe - unicode" {
    try testCountFromPipe("hélló", .{ .bytes = 7, .lines = 0, .words = 1, .chars = 5 });
}
