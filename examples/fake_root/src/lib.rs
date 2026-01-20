#[dlhook::dlhook(origin = "getuid")]
fn fake_root_uid(_: _) -> u32 {
    0
}

#[dlhook::dlhook(origin = "geteuid")]
fn fake_root_euid(_: _) -> u32 {
    0
}
