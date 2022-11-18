use bitvec::{order::Lsb0, vec::BitVec};
use egui::{Color32, Label, RichText, TextEdit};
use egui_miniquad as egui_mq;
use hamming::{code::EH16_11, decode, encode, Code};
use miniquad as mq;
use rand::{distributions::Uniform, thread_rng, Rng};
use std::str::from_utf8;

struct Stage {
    egui_mq: egui_mq::EguiMq,
    message_in: String,
    encoded: Vec<u8>,
    error: Vec<u8>,
    with_error: Vec<u8>,
    message_out: Option<String>,
    code: Code,
    prob_str: String,
}

impl Stage {
    fn new(ctx: &mut mq::Context) -> Self {
        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            message_in: String::new(),
            encoded: Vec::new(),
            error: Vec::new(),
            with_error: Vec::new(),
            message_out: Some(String::new()),
            code: EH16_11,
            prob_str: String::new(),
        }
    }
}

impl mq::EventHandler for Stage {
    fn update(&mut self, _ctx: &mut mq::Context) {}

    fn draw(&mut self, mq_ctx: &mut mq::Context) {
        mq_ctx.clear(Some((1., 1., 1., 1.)), None, None);
        mq_ctx.begin_default_pass(mq::PassAction::clear_color(0.2, 0.2, 0.2, 1.0));
        mq_ctx.end_render_pass();

        self.encoded = encode(self.message_in.as_bytes(), &self.code).unwrap();
        self.error.resize_with(self.encoded.len(), || 0);

        self.with_error.clear();
        for (b, e) in Iterator::zip(self.encoded.iter(), self.error.iter()) {
            self.with_error.push(b ^ e);
        }
        self.message_out = decode(&self.with_error, &self.code)
            .ok()
            .and_then(|decoded| from_utf8(&decoded).ok().map(String::from));

        self.egui_mq.run(mq_ctx, |_mq_ctx, egui_ctx| {
            egui::TopBottomPanel::top("set_error").show(egui_ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Error Probability:");
                    ui.text_edit_singleline(&mut self.prob_str);
                    let enabled = self
                        .prob_str
                        .parse::<f64>()
                        .map_or(false, |prob| (0.0..=1.0).contains(&prob));
                    if ui
                        .add_enabled(enabled, egui::Button::new("Randomize Error"))
                        .clicked()
                    {
                        let prob = self.prob_str.parse::<f64>().unwrap();
                        let mut bits = BitVec::<u8, Lsb0>::from_vec(self.error.clone());

                        let mut rng = thread_rng();
                        let distr = Uniform::new(0.0, 1.0);
                        for mut bit in &mut bits {
                            bit.set(rng.sample(distr) < prob);
                        }
                        self.error = bits.into_vec();
                    }
                })
            });

            egui::SidePanel::left("original").show(egui_ctx, |ui| {
                ui.label("Original");

                let m = TextEdit::multiline(&mut self.message_in);
                ui.add(m);
            });

            egui::SidePanel::left("encoded").show(egui_ctx, |ui| {
                ui.label("Encoded");

                let mut s = String::new();
                for b in &self.encoded {
                    s.push_str(&format!("{b:08b}\n"));
                }

                let m = TextEdit::multiline(&mut s).interactive(false);
                ui.add(m);
            });

            egui::SidePanel::left("error").show(egui_ctx, |ui| {
                ui.label("Error");

                let mut s = String::new();
                for b in &self.error {
                    s.push_str(&format!("{b:08b}\n"));
                }

                let m = TextEdit::multiline(&mut s).interactive(false);
                ui.add(m);
            });

            egui::SidePanel::left("with_error").show(egui_ctx, |ui| {
                ui.label("Encoded with Error");

                let mut s = String::new();
                for b in &self.with_error {
                    s.push_str(&format!("{b:08b}\n"));
                }

                let m = TextEdit::multiline(&mut s).interactive(false);
                ui.add(m);
            });

            egui::SidePanel::left("decoded").show(egui_ctx, |ui| {
                ui.label("Decoded");

                match &mut self.message_out {
                    Some(message_out) => {
                        let m = TextEdit::multiline(message_out).interactive(false);
                        ui.add(m);
                    }
                    None => {
                        let l = Label::new(
                            RichText::new("Unable to decode message.").color(Color32::RED),
                        );
                        ui.add(l);
                    }
                };
            });
        });

        self.egui_mq.draw(mq_ctx);
        mq_ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, _: &mut mq::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, _: &mut mq::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut mq::Context,
        mb: mq::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut mq::Context,
        character: char,
        _keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut mq::Context,
        keycode: mq::KeyCode,
        keymods: mq::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(&mut self, _ctx: &mut mq::Context, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

fn main() {
    let conf = mq::conf::Conf {
        window_title: "Hamming".to_string(),
        high_dpi: true,
        ..Default::default()
    };
    mq::start(conf, |ctx| Box::new(Stage::new(ctx)));
}
