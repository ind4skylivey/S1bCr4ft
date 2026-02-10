/// Security hardening presets
pub struct SecurityPresets;

impl SecurityPresets {
    pub fn kernel_hardening() -> Vec<(&'static str, &'static str)> {
        vec![
            ("kernel.kptr_restrict", "2"),
            ("kernel.dmesg_restrict", "1"),
            ("kernel.unprivileged_bpf_disabled", "1"),
            ("net.ipv4.conf.all.rp_filter", "1"),
            ("net.ipv4.conf.default.rp_filter", "1"),
        ]
    }

    pub fn apparmor_profiles() -> Vec<&'static str> {
        vec![
            "/etc/apparmor.d/usr.bin.firefox",
            "/etc/apparmor.d/usr.bin.chromium",
        ]
    }
}
