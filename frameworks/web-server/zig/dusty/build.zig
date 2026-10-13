const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const exe = b.addExecutable(.{
        .name = "server",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const dusty = b.dependency("dusty", .{
        .target = target,
        .optimize = optimize,
        .use_tls = false,
    });
    exe.root_module.addImport("dusty", dusty.module("dusty"));

    b.installArtifact(exe);
}
