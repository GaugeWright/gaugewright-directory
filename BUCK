# This repository's one stage-1 Buck2 target: its green bar, as an opaque action
# over the whole checkout (GaugeWright BUILD.md, DR-0124). The rule is owned by
# the gaugewright cell; this file is inert until a workspace's generated
# .buckconfig names this checkout as the `gaugewright-directory` cell.
load("@gaugewright//tools/buck:check.bzl", "check")

check(
    name = "check",
    checkout = "gaugewright-directory",
    srcs = glob(["**/*"]),
)
