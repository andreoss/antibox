    use super::*;
    use crate::id::ClientId;
    use antibox_core::mock::MockDisplay;

    fn shot(w: u16, h: u16) -> Vec<u8> {
        let mut data = vec![0u8; w as usize * h as usize * 4];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let i = (y * w as usize + x) * 4;
                data[i] = if x < w as usize / 2 { 0 } else { 0xff };
                data[i + 1] = 0x40;
                data[i + 2] = if y < h as usize / 2 { 0 } else { 0xff };
                data[i + 3] = 0xff;
            }
        }
        data
    }

    fn tile(client: u32, cell: Cell) -> Tile {
        Tile {
            client: ClientId(u64::from(client)),
            title: format!("window {client}"),
            cell,
            art: Art::Thumb(Vec::new()),
            dim: None,
        }
    }

    fn with_tiles(cells: &[Cell]) -> Expose {
        let mut e = Expose::new();
        e.tiles = cells
            .iter()
            .enumerate()
            .map(|(i, c)| tile(i as u32 + 1, *c))
            .collect();
        e.visible = true;
        e.area = Rect::new(0, 0, 1280, 720);
        e
    }

    fn grid() -> Expose {
        with_tiles(&[
            Cell { x: 0, y: 0, w: 100, h: 100 },
            Cell { x: 200, y: 0, w: 100, h: 100 },
            Cell { x: 0, y: 200, w: 100, h: 100 },
        ])
    }

    #[test]
    fn test_crop_scale_averages_the_source_block() {
        let src = shot(64, 64);
        let out = crop_scale(&src, 64, 64, Rect::new(0, 0, 64, 64), 2, 2).unwrap();
        assert_eq!(out.len(), 2 * 2 * 4);
        assert_eq!(out[0], 0x00);
        assert_eq!(out[4], 0xff);
        assert_eq!(out[2], 0x00);
        assert_eq!(out[8 + 2], 0xff);
        assert!(out.chunks_exact(4).all(|px| px[3] == 0xff));
    }

    #[test]
    fn test_crop_scale_clamps_to_the_screen() {
        let src = shot(64, 64);
        let out = crop_scale(&src, 64, 64, Rect::new(-20, -20, 64, 64), 4, 4).unwrap();
        assert_eq!(out.len(), 4 * 4 * 4);
        assert!(crop_scale(&src, 64, 64, Rect::new(100, 100, 40, 40), 4, 4).is_none());
        assert!(crop_scale(&src, 64, 64, Rect::new(0, 0, 10, 10), 0, 4).is_none());
        assert!(crop_scale(&[], 64, 64, Rect::new(0, 0, 10, 10), 4, 4).is_none());
    }

    #[test]
    fn test_keys_move_the_selection() {
        let mut e = grid();
        assert_eq!(e.selected(), 0);
        assert_eq!(e.handle_key(KEY_RIGHT, false), ExposeAction::Repaint);
        assert_eq!(e.selected(), 1);
        assert_eq!(e.handle_key(KEY_RIGHT, false), ExposeAction::None);
        assert_eq!(e.handle_key(0x0068, false), ExposeAction::Repaint);
        assert_eq!(e.selected(), 0);
        assert_eq!(e.handle_key(KEY_DOWN, false), ExposeAction::Repaint);
        assert_eq!(e.selected(), 2);
        assert_eq!(e.handle_key(0x006b, false), ExposeAction::Repaint);
        assert_eq!(e.selected(), 0);
    }

    #[test]
    fn test_tab_cycles_both_ways() {
        let mut e = grid();
        e.handle_key(KEY_TAB, false);
        assert_eq!(e.selected(), 1);
        e.handle_key(KEY_TAB, false);
        e.handle_key(KEY_TAB, false);
        assert_eq!(e.selected(), 0);
        e.handle_key(KEY_TAB, true);
        assert_eq!(e.selected(), 2);
    }

    #[test]
    fn test_enter_activates_and_escape_cancels() {
        let mut e = grid();
        e.handle_key(KEY_TAB, false);
        let want = e.selected_client().unwrap();
        assert_eq!(e.handle_key(KEY_RETURN, false), ExposeAction::Activate(want));
        assert_eq!(e.handle_key(KEY_SPACE, false), ExposeAction::Activate(want));
        assert_eq!(e.handle_key(KEY_ESCAPE, false), ExposeAction::Cancel);
        assert_eq!(e.handle_key(0x0071, false), ExposeAction::Cancel);
        assert_eq!(e.handle_key(0x0041, false), ExposeAction::None);
    }

    #[test]
    fn test_keys_without_tiles_cancel() {
        let mut e = Expose::new();
        assert_eq!(e.handle_key(KEY_DOWN, false), ExposeAction::Cancel);
    }

    #[test]
    fn test_pointer_selects_and_clicks_activate() {
        let mut e = grid();
        assert_eq!(e.handle_motion(Point::new(250, 50)), ExposeAction::Repaint);
        assert_eq!(e.selected(), 1);
        assert_eq!(e.handle_motion(Point::new(260, 60)), ExposeAction::Repaint);
        assert_eq!(e.handle_motion(Point::new(150, 150)), ExposeAction::Repaint);
        e.set_prefs(ExposePrefs { tooltip_follows_mouse: false, ..ExposePrefs::default() });
        assert_eq!(e.handle_motion(Point::new(262, 62)), ExposeAction::None);
        let want = e.tiles()[2].client;
        assert_eq!(
            e.handle_click(Point::new(50, 250), 1),
            ExposeAction::Activate(want)
        );
        assert_eq!(e.handle_click(Point::new(600, 600), 1), ExposeAction::Cancel);
    }

    #[test]
    fn test_hide_clears_state() {
        let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut e = grid();
        e.hide(&conn);
        assert!(!e.visible);
        assert!(e.tiles().is_empty());
        assert_eq!(e.selected(), 0);
        assert!(!e.owns_window(1));
    }

    #[test]
    fn test_draw_does_not_panic() {
        let g = antibox_core::mock::MockGraphics::new(1);
        let e = grid();
        e.draw(&g, 1280, 720);
    }

    fn insert_client(wm: &mut WindowManager<dyn DisplayBackend>, rect: Rect) -> ClientId {
        let d = MockDisplay::new(1280, 720, 24);
        let win = |r: Rect| {
            d.create_window(1, r, WmWindowClass::InputOutput, false, EventMask::NO_EVENT)
                .unwrap()
        };
        let client_win = win(rect);
        let frame_win = win(rect);
        let client_xid = client_win.id();
        let frame_xid = frame_win.id();
        let mut fw =
            crate::frame::FrameWindow::new(crate::client::ClientWindow::new(client_win), frame_win);
        fw.set_frame_rect(rect);
        let cid = fw.client_id();
        wm.xid_index.insert(cid, client_xid, frame_xid);
        wm.frames.insert(cid, fw);
        cid
    }

    fn built(wm: &WindowManager<dyn DisplayBackend>, prefs: &ExposePrefs) -> Vec<ClientId> {
        Expose::build_tiles(wm, Rect::new(0, 0, 1280, 720), None, prefs).tiles
            .iter()
            .map(|t| t.client)
            .collect()
    }

    #[test]
    fn test_build_tiles_honours_the_client_filters() {
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        let plain = insert_client(&mut wm, Rect::new(0, 0, 400, 300));
        let other_ws = insert_client(&mut wm, Rect::new(10, 10, 400, 300));
        let skipped = insert_client(&mut wm, Rect::new(20, 20, 400, 300));
        let hidden = insert_client(&mut wm, Rect::new(30, 30, 400, 300));
        wm.frame_mut(other_ws).unwrap().set_workspace(1);
        wm.frame_mut(skipped).unwrap().state_mut().skip_taskbar = true;
        wm.frame_mut(hidden).unwrap().state_mut().minimized = true;

        let mut prefs = ExposePrefs::default();
        assert_eq!(built(&wm, &prefs), vec![plain, other_ws, hidden]);

        prefs.all_desktops = false;
        assert_eq!(built(&wm, &prefs), vec![plain, hidden]);

        prefs.all_desktops = true;
        assert_eq!(built(&wm, &prefs), vec![plain, other_ws, hidden]);

        prefs.show_skip_taskbar = true;
        assert_eq!(built(&wm, &prefs), vec![plain, other_ws, skipped, hidden]);

        prefs.show_minimized = false;
        assert_eq!(built(&wm, &prefs), vec![plain, other_ws, skipped]);
    }

    #[test]
    fn test_build_tiles_shrinks_the_cells_as_distance_grows() {
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        insert_client(&mut wm, Rect::new(0, 0, 600, 400));
        insert_client(&mut wm, Rect::new(700, 0, 600, 400));
        let width_of = |p: &ExposePrefs| Expose::build_tiles(&wm, Rect::new(0, 0, 1280, 720), None, p).tiles[0].cell.w;
        let tight = width_of(&ExposePrefs { distance: 1, ..ExposePrefs::default() });
        let loose = width_of(&ExposePrefs { distance: 400, ..ExposePrefs::default() });
        assert!(loose < tight, "{loose} {tight}");
    }

    #[test]
    fn test_mode_words_parse_in_order() {
        assert_eq!(
            DispMode::parse_list("icon thumbnail"),
            vec![DispMode::Icon, DispMode::Thumbnail]
        );
        assert_eq!(
            DispMode::parse_list("icon bogus filled"),
            vec![DispMode::Icon, DispMode::Filled]
        );
        assert_eq!(
            DispMode::parse_list("ICON Thumbnail-Icon"),
            vec![DispMode::Icon, DispMode::ThumbnailIcon]
        );
        let fallback = DispMode::parse_list(crate::wmconfig::DEFAULT_EXPOSE_MODES);
        assert_eq!(DispMode::parse_list(""), fallback);
        assert_eq!(DispMode::parse_list("nonsense"), fallback);
        assert_eq!(
            fallback,
            vec![
                DispMode::Thumbnail,
                DispMode::Icon,
                DispMode::Filled,
                DispMode::None
            ]
        );
    }

    #[test]
    fn test_mode_chain_falls_back() {
        let chain = DispMode::parse_list(crate::wmconfig::DEFAULT_EXPOSE_MODES);
        assert_eq!(pick_mode(&chain, true, true), DispMode::Thumbnail);
        assert_eq!(pick_mode(&chain, false, true), DispMode::Icon);
        assert_eq!(pick_mode(&chain, false, false), DispMode::Filled);

        let chain = DispMode::parse_list("thumbnail-icon thumbnail none");
        assert_eq!(pick_mode(&chain, true, true), DispMode::ThumbnailIcon);
        assert_eq!(pick_mode(&chain, true, false), DispMode::Thumbnail);
        assert_eq!(pick_mode(&chain, false, true), DispMode::None);
        assert_eq!(pick_mode(&[], true, true), DispMode::None);
    }

    fn give_icon(wm: &mut WindowManager<dyn DisplayBackend>, id: ClientId) {
        wm.frame_mut(id).unwrap().client_mut().icon = vec![IconData {
            width: 2,
            height: 2,
            pixels: vec![0xffff_ffff; 4],
        }];
    }

    fn art_of(wm: &WindowManager<dyn DisplayBackend>, modes: &str, shot: bool) -> Vec<&'static str> {
        let screen = shot.then(|| (1280u16, 720u16, self::shot(1280, 720)));
        let prefs = ExposePrefs { modes: modes.to_string(), ..ExposePrefs::default() };
        Expose::build_tiles(wm, Rect::new(0, 0, 1280, 720), screen.as_ref(), &prefs).tiles
            .iter()
            .map(|t| match t.art {
                Art::Thumb(_) => "thumb",
                Art::ThumbIcon(_, _) => "thumb-icon",
                Art::Icon(_) => "icon",
                Art::Filled => "filled",
            })
            .collect()
    }

    #[test]
    fn test_art_follows_the_mode_chain() {
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        let with = insert_client(&mut wm, Rect::new(0, 0, 400, 300));
        insert_client(&mut wm, Rect::new(500, 0, 400, 300));
        give_icon(&mut wm, with);

        assert_eq!(art_of(&wm, "thumbnail icon filled none", true), ["thumb", "thumb"]);
        assert_eq!(art_of(&wm, "thumbnail icon filled none", false), ["icon", "filled"]);
        assert_eq!(art_of(&wm, "thumbnail-icon icon filled", true), ["thumb-icon", "filled"]);
        assert_eq!(art_of(&wm, "icon filled", false), ["icon", "filled"]);
    }

    #[test]
    fn test_clients_without_art_leave_the_layout() {
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        let with = insert_client(&mut wm, Rect::new(0, 0, 400, 300));
        insert_client(&mut wm, Rect::new(500, 0, 400, 300));
        give_icon(&mut wm, with);

        assert_eq!(art_of(&wm, "thumbnail none", false), Vec::<&str>::new());
        assert_eq!(art_of(&wm, "icon none", false), ["icon"]);
        assert_eq!(art_of(&wm, "thumbnail none", true), ["thumb", "thumb"]);
    }

    #[test]
    fn test_draw_covers_every_art() {
        let g = antibox_core::mock::MockGraphics::new(1);
        let mut e = grid();
        let icon = crate::icon_render::default_icon(16, 0);
        e.tiles[0].art = Art::Filled;
        e.tiles[1].art = Art::Icon(icon.clone());
        e.tiles[2].art = Art::ThumbIcon(vec![0u8; 100 * 100 * 4], icon);
        e.draw(&g, 1280, 720);
    }

    #[test]
    fn test_client_ops_parse() {
        assert_eq!(ClientOp::parse("focus"), Some(ClientOp::Focus));
        assert_eq!(ClientOp::parse(" Iconify "), Some(ClientOp::Iconify));
        assert_eq!(ClientOp::parse("minimize"), Some(ClientOp::Iconify));
        assert_eq!(ClientOp::parse("shade"), Some(ClientOp::Shade));
        assert_eq!(ClientOp::parse("close"), Some(ClientOp::Close));
        assert_eq!(ClientOp::parse("destroy"), Some(ClientOp::Kill));
        assert_eq!(ClientOp::parse("no"), Some(ClientOp::None));
        assert_eq!(ClientOp::parse("wiggle"), None);
    }

    #[test]
    fn test_buttons_run_their_bound_op() {
        let mut e = grid();
        let first = e.tiles()[0].client;
        assert_eq!(e.handle_click(Point::new(50, 50), 1), ExposeAction::Activate(first));
        assert_eq!(
            e.handle_click(Point::new(50, 50), 2),
            ExposeAction::Client(ClientOp::Close, first)
        );
        assert_eq!(
            e.handle_click(Point::new(50, 50), 3),
            ExposeAction::Client(ClientOp::Iconify, first)
        );
        assert_eq!(e.handle_click(Point::new(50, 50), 4), ExposeAction::None);
        assert_eq!(e.handle_click(Point::new(700, 700), 2), ExposeAction::Cancel);
    }

    #[test]
    fn test_unbound_and_custom_buttons() {
        let mut e = grid();
        e.set_prefs(ExposePrefs {
            mouse1: "shade".to_string(),
            mouse2: "no".to_string(),
            mouse3: "wiggle".to_string(),
            ..ExposePrefs::default()
        });
        let first = e.tiles()[0].client;
        assert_eq!(
            e.handle_click(Point::new(50, 50), 1),
            ExposeAction::Client(ClientOp::Shade, first)
        );
        assert_eq!(e.handle_click(Point::new(50, 50), 2), ExposeAction::None);
        assert_eq!(e.handle_click(Point::new(50, 50), 3), ExposeAction::None);
    }

    #[test]
    fn test_rebuild_keeps_the_selected_client() {
        let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        let a = insert_client(&mut wm, Rect::new(0, 0, 400, 300));
        let b = insert_client(&mut wm, Rect::new(500, 0, 400, 300));
        let mut e = Expose::new();
        e.visible = true;
        e.area = Rect::new(0, 0, 1280, 720);
        e.tiles = Expose::build_tiles(&wm, e.area, None, &e.prefs).tiles;
        e.selected = 1;
        assert_eq!(e.selected_client(), Some(b));

        wm.frames.remove(&a);
        e.rebuild(&conn, &wm);
        assert!(e.visible);
        assert_eq!(e.tiles().len(), 1);
        assert_eq!(e.selected_client(), Some(b));

        wm.frames.remove(&b);
        e.rebuild(&conn, &wm);
        assert!(!e.visible);
        assert!(e.tiles().is_empty());
    }

    #[test]
    fn test_tile_style_comes_from_prefs() {
        let prefs = ExposePrefs::default();
        let normal = TileStyle::of(&prefs, false);
        let high = TileStyle::of(&prefs, true);
        assert_eq!(normal.opacity, 150);
        assert_eq!(normal.tint_opacity, 0);
        assert_eq!(high.opacity, 255);
        assert_eq!(high.tint_opacity, 0);
        assert!(!normal.is_neutral());
        assert!(high.is_neutral());
        assert!(TileStyle { tint: 0, tint_opacity: 0, opacity: 255 }.is_neutral());
    }

    #[test]
    fn test_style_mixes_towards_tint_and_backdrop() {
        let opaque = TileStyle { tint: 0x0000_0000, tint_opacity: 255, opacity: 255 };
        assert_eq!(opaque.apply(0x00ff_ffff, 0x00ff_0000), 0x0000_0000);
        let clear = TileStyle { tint: 0, tint_opacity: 0, opacity: 0 };
        assert_eq!(clear.apply(0x00ff_ffff, 0x0012_3456), 0x0012_3456);
        let half = TileStyle { tint: 0, tint_opacity: 0, opacity: 128 };
        let mixed = half.apply(0x00ff_ffff, 0x0000_0000);
        assert!((0x7e..=0x82).contains(&(mixed & 0xff)), "{mixed:#08x}");
    }

    #[test]
    fn test_blend_pixels_skips_neutral_styles() {
        let src = shot(4, 4);
        let neutral = TileStyle { tint: 0, tint_opacity: 0, opacity: 255 };
        assert!(blend_pixels(&src, neutral, 0x0033_3333).is_none());

        let dark = TileStyle { tint: 0, tint_opacity: 255, opacity: 255 };
        let out = blend_pixels(&src, dark, 0x0033_3333).unwrap();
        assert_eq!(out.len(), src.len());
        assert!(out.chunks_exact(4).all(|px| px[0] == 0 && px[1] == 0 && px[2] == 0));

        let faded = TileStyle { tint: 0, tint_opacity: 0, opacity: 0 };
        let out = blend_pixels(&src, faded, 0x0033_4455).unwrap();
        assert!(out
            .chunks_exact(4)
            .all(|px| px[0] == 0x55 && px[1] == 0x44 && px[2] == 0x33));
    }

    #[test]
    fn test_tip_follows_the_pointer_with_offsets() {
        let mut e = grid();
        e.handle_motion(Point::new(400, 300));
        let cell = e.tiles()[0].cell;
        let tile = tile(1, cell);
        let (x, y) = e.tip_origin(&tile, 100, 20, 1280);
        assert_eq!((x, y), (420, 320));

        e.set_prefs(ExposePrefs {
            tooltip_align: "mid".to_string(),
            tooltip_offset_x: 0,
            tooltip_offset_y: -40,
            ..ExposePrefs::default()
        });
        let (x, y) = e.tip_origin(&tile, 100, 20, 1280);
        assert_eq!((x, y), (350, 260));

        e.set_prefs(ExposePrefs {
            tooltip_align: "right".to_string(),
            tooltip_offset_x: 0,
            tooltip_offset_y: 0,
            ..ExposePrefs::default()
        });
        assert_eq!(e.tip_origin(&tile, 100, 20, 1280).0, 300);
    }

    #[test]
    fn test_tip_stays_on_screen() {
        let mut e = grid();
        e.handle_motion(Point::new(1270, 710));
        let tile = tile(1, e.tiles()[0].cell);
        let (x, y) = e.tip_origin(&tile, 200, 30, 1280);
        assert_eq!(x, 1080);
        assert_eq!(y, 690);
    }

    #[test]
    fn test_tip_sits_under_the_tile_when_not_following() {
        let mut e = grid();
        e.set_prefs(ExposePrefs { tooltip_follows_mouse: false, ..ExposePrefs::default() });
        let tile = tile(2, e.tiles()[1].cell);
        let (x, y) = e.tip_origin(&tile, 60, 20, 1280);
        assert_eq!(x, 220);
        assert!(y >= 100, "{y}");
    }

    #[test]
    fn test_keys_aim_the_pointer_when_asked() {
        let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut e = grid();
        e.set_prefs(ExposePrefs { warp_on_select: true, ..ExposePrefs::default() });
        assert_eq!(e.handle_key(KEY_RIGHT, false), ExposeAction::Repaint);
        assert_eq!(e.warp_to, Some(Point::new(250, 50)));
        e.pump(&conn);
        assert_eq!(e.warp_to, None);
        assert_eq!(e.pointer, Point::new(250, 50));

        e.handle_key(KEY_TAB, false);
        assert!(e.warp_to.is_some());
        e.set_prefs(ExposePrefs::default());
        e.warp_to = None;
        e.handle_key(KEY_TAB, false);
        assert_eq!(e.warp_to, None);
    }

    #[test]
    fn test_hide_forgets_a_pending_warp() {
        let conn: Arc<dyn DisplayBackend> = Arc::new(MockDisplay::new(1280, 720, 24));
        let mut e = grid();
        e.warp_to = Some(Point::new(10, 10));
        e.hide(&conn);
        assert_eq!(e.warp_to, None);
    }

    #[test]
    fn test_monitor_lookup_and_membership() {
        let monitors = vec![
            MonitorInfo { x: 0, y: 0, width: 1280, height: 800 },
            MonitorInfo { x: 1280, y: 0, width: 1920, height: 1080 },
        ];
        let left = Rect::new(0, 0, 1280, 800);
        let right = Rect::new(1280, 0, 1920, 1080);
        assert_eq!(monitor_at(&monitors, Point::new(10, 10)), Some(left));
        assert_eq!(monitor_at(&monitors, Point::new(1500, 500)), Some(right));
        assert_eq!(monitor_at(&monitors, Point::new(4000, 500)), None);
        assert_eq!(monitor_at(&[], Point::ZERO), None);

        assert!(on_monitor(Rect::new(20, 20, 200, 100), left));
        assert!(!on_monitor(Rect::new(20, 20, 200, 100), right));
        assert!(on_monitor(Rect::new(1200, 0, 400, 300), right));
    }

    #[test]
    fn test_one_monitor_keeps_its_own_clients() {
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        let here = insert_client(&mut wm, Rect::new(40, 40, 400, 300));
        let there = insert_client(&mut wm, Rect::new(1400, 40, 400, 300));
        let left = Rect::new(0, 0, 1280, 800);

        let all = ExposePrefs::default();
        let ids: Vec<ClientId> = Expose::build_tiles(&wm, left, None, &all).tiles
            .iter()
            .map(|t| t.client)
            .collect();
        assert_eq!(ids, vec![here, there]);

        let one = ExposePrefs { all_monitors: false, ..ExposePrefs::default() };
        let ids: Vec<ClientId> = Expose::build_tiles(&wm, left, None, &one).tiles
            .iter()
            .map(|t| t.client)
            .collect();
        assert_eq!(ids, vec![here]);
    }

    #[test]
    fn test_cells_stay_inside_the_monitor() {
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        insert_client(&mut wm, Rect::new(1400, 40, 900, 700));
        insert_client(&mut wm, Rect::new(1500, 90, 900, 700));
        let right = Rect::new(1280, 0, 1920, 1080);
        let prefs = ExposePrefs { all_monitors: false, ..ExposePrefs::default() };
        for tile in Expose::build_tiles(&wm, right, None, &prefs).tiles {
            assert!(tile.cell.x >= 0, "{:?}", tile.cell);
            assert!(tile.cell.y >= 0, "{:?}", tile.cell);
            assert!(tile.cell.x + tile.cell.w <= right.w, "{:?}", tile.cell);
            assert!(tile.cell.y + tile.cell.h <= right.h, "{:?}", tile.cell);
        }
    }

    #[test]
    fn test_live_pictures_beat_the_screen_grab() {
        let mut wm = WindowManager::<dyn DisplayBackend>::new_test();
        insert_client(&mut wm, Rect::new(0, 0, 400, 300));
        let prefs = ExposePrefs { live_thumbnails: true, ..ExposePrefs::default() };
        let built = Expose::build_tiles(&wm, Rect::new(0, 0, 1280, 720), None, &prefs);
        assert_eq!(built.missing_live, 1);
        assert!(matches!(built.tiles[0].art, Art::Filled));

        let off = ExposePrefs { live_thumbnails: false, ..ExposePrefs::default() };
        let built = Expose::build_tiles(&wm, Rect::new(0, 0, 1280, 720), None, &off);
        assert_eq!(built.missing_live, 0);
    }
