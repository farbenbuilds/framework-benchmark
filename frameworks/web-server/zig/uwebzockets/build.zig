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
    const uz = b.dependency("uz", .{
        .target = target,
        .optimize = optimize,
        .debug_logs = false,
        .ssl = false,
        .no_zlib = true,
        .with_proxy = false,
        .with_uv = false,
    });

    exe.root_module.addImport("uz", uz.module("zuws"));
    b.installArtifact(exe);
}
