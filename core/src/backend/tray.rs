pub trait TrayBackend: Send + Sync {
    fn composite_redirect(&self, client: u32);

    fn composite_unredirect(&self, client: u32);

    fn create_damage(&self, client: u32) -> Option<u32>;

    fn destroy_damage(&self, damage: u32);

    fn get_xembed_info(&self, client: u32, xembed_info_atom: u32) -> Option<(u32, bool)>;

    fn get_text_property(&self, window: u32, atom: u32) -> Option<String>;

    fn send_xembed(&self, target: u32, xembed_atom: u32, data: [u32; 5]);

    fn flush(&self);
}
