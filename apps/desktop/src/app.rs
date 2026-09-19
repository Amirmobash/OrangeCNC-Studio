use std::fs;

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Vec2};
use orangecnc_domain::{CanonicalCommand, Vec3, AUTHOR, PRODUCT_NAME};
use orangecnc_gcode::Parser;
use orangecnc_machine::MachineProfile;
use orangecnc_motion::{MotionPlanner, PlannerSettings, SegmentKind, Toolpath};
use orangecnc_simulation::{SimulationReport, Simulator};

use crate::theme::{self, BORDER, MUTED, ORANGE, ORANGE_SOFT, TEXT};

const DEMO: &str = r#"; OrangeCNC Demo – Amir Mobasheraghdam
G21 G90
G0 X20 Y20 Z0
G1 X100 Y20 F1500
G2 X100 Y100 I0 J40
G1 X20 Y100
G3 X20 Y20 I0 J-40
M5
M30
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode { Top, Isometric }

pub struct OrangeCncApp {
    source: String,
    status: String,
    toolpath: Toolpath,
    report: SimulationReport,
    view_mode: ViewMode,
    machine: MachineProfile,
}

impl OrangeCncApp {
    pub fn new(cc:&eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let mut app=Self {
            source: DEMO.into(), status:"Bereit".into(), toolpath:Toolpath::default(),
            report:SimulationReport::default(), view_mode:ViewMode::Top,
            machine:MachineProfile::default(),
        };
        app.analyze();
        app
    }

    fn analyze(&mut self) {
        let mut parser=Parser::new();
        let parsed=match parser.parse_program(&self.source) {
            Ok(lines)=>lines,
            Err(err)=>{ self.status=format!("Parserfehler: {err}"); return; }
        };
        let commands:Vec<CanonicalCommand>=parsed.into_iter().flat_map(|line| line.commands).collect();
        let planner=MotionPlanner::new(PlannerSettings::default());
        self.toolpath=match planner.build(&commands) {
            Ok(path)=>path,
            Err(err)=>{ self.status=format!("Geometriefehler: {err}"); return; }
        };
        self.report=Simulator::new(self.machine.clone()).run(&self.toolpath);
        self.status=if self.report.warnings.is_empty() {
            format!("Analyse OK · {} Segmente · {:.2} mm",self.report.segment_count,self.report.total_length_mm)
        } else {
            format!("Analyse mit {} Warnung(en)",self.report.warnings.len())
        };
    }

    fn open_file(&mut self) {
        let Some(path)=rfd::FileDialog::new().add_filter("G-Code", &["nc","gcode","ngc","tap"]).pick_file() else { return; };
        match fs::read_to_string(&path) {
            Ok(source)=>{ self.source=source; self.status=format!("Geladen: {}",path.display()); self.analyze(); }
            Err(err)=>self.status=format!("Dateifehler: {err}"),
        }
    }

    fn save_file(&mut self) {
        let Some(path)=rfd::FileDialog::new().set_file_name("programm.nc").save_file() else { return; };
        match fs::write(&path,&self.source) {
            Ok(())=>self.status=format!("Gespeichert: {}",path.display()),
            Err(err)=>self.status=format!("Dateifehler: {err}"),
        }
    }

    fn draw_path(&self, ui:&mut egui::Ui) {
        let desired=ui.available_size().max(Vec2::new(320.0,300.0));
        let (response,painter)=ui.allocate_painter(desired,Sense::hover());
        let rect=response.rect.shrink(18.0);
        painter.rect_filled(rect,10.0,Color32::WHITE);
        painter.rect_stroke(rect,10.0,Stroke::new(1.0,BORDER),egui::StrokeKind::Inside);
        let Some((min,max))=self.toolpath.bounds() else {
            painter.text(rect.center(),Align2::CENTER_CENTER,"Keine Werkzeugbahn",FontId::proportional(18.0),MUTED);
            return;
        };
        let project=|p:Vec3| -> (f64,f64) {
            match self.view_mode {
                ViewMode::Top => (p.x,p.y),
                ViewMode::Isometric => (p.x-p.y,(p.x+p.y)*0.45-p.z*1.2),
            }
        };
        let corners=[Vec3::new(min.x,min.y,min.z),Vec3::new(max.x,min.y,min.z),Vec3::new(min.x,max.y,min.z),Vec3::new(max.x,max.y,max.z)];
        let pts:Vec<(f64,f64)>=corners.into_iter().map(project).collect();
        let min_u=pts.iter().map(|p|p.0).fold(f64::INFINITY,f64::min);
        let max_u=pts.iter().map(|p|p.0).fold(f64::NEG_INFINITY,f64::max);
        let min_v=pts.iter().map(|p|p.1).fold(f64::INFINITY,f64::min);
        let max_v=pts.iter().map(|p|p.1).fold(f64::NEG_INFINITY,f64::max);
        let map=|p:Vec3| -> Pos2 {
            let (u,v)=project(p); let du=(max_u-min_u).max(1e-6); let dv=(max_v-min_v).max(1e-6);
            let scale=(rect.width() as f64/du).min(rect.height() as f64/dv)*0.9;
            let cx=(min_u+max_u)/2.0; let cy=(min_v+max_v)/2.0;
            Pos2::new(rect.center().x+((u-cx)*scale) as f32,rect.center().y-((v-cy)*scale) as f32)
        };
        for segment in &self.toolpath.segments {
            let color=match segment.kind { SegmentKind::Rapid=>Color32::from_rgb(150,150,150), SegmentKind::Feed=>ORANGE, SegmentKind::Arc=>Color32::from_rgb(220,90,20) };
            painter.line_segment([map(segment.start),map(segment.end)],Stroke::new(2.0,color));
        }
        painter.text(rect.left_top()+Vec2::new(8.0,8.0),Align2::LEFT_TOP,match self.view_mode {ViewMode::Top=>"Draufsicht",ViewMode::Isometric=>"Isometrisch"},FontId::proportional(14.0),MUTED);
    }
}

impl eframe::App for OrangeCncApp {
    fn update(&mut self,ctx:&egui::Context,_frame:&mut eframe::Frame) {
        egui::TopBottomPanel::top("header").exact_height(74.0).show(ctx,|ui|{
            let rect=ui.max_rect(); ui.painter().rect_filled(rect,0.0,ORANGE);
            ui.horizontal(|ui|{
                ui.add_space(18.0); ui.vertical(|ui|{
                    ui.add_space(10.0); ui.label(egui::RichText::new(PRODUCT_NAME).size(25.0).strong().color(Color32::WHITE));
                    ui.label(egui::RichText::new(format!("CNC-Workbench · {AUTHOR}")).size(13.0).color(Color32::from_rgb(255,236,215)));
                });
            });
        });
        egui::SidePanel::left("sidebar").resizable(false).exact_width(250.0).frame(egui::Frame::new().fill(ORANGE_SOFT).inner_margin(egui::Margin::same(14))).show(ctx,|ui|{
            ui.heading("Projekt"); ui.add_space(8.0);
            if ui.button("📂  G-Code öffnen").clicked(){self.open_file();}
            if ui.button("💾  Speichern").clicked(){self.save_file();}
            if ui.button("▶  Analysieren").clicked(){self.analyze();}
            if ui.button("↺  Demo laden").clicked(){self.source=DEMO.into();self.analyze();}
            ui.separator(); ui.heading("Ansicht");
            ui.selectable_value(&mut self.view_mode,ViewMode::Top,"Draufsicht");
            ui.selectable_value(&mut self.view_mode,ViewMode::Isometric,"Isometrisch");
            ui.separator(); ui.heading("Maschine");
            ui.label(egui::RichText::new(&self.machine.name).color(TEXT));
            ui.label(egui::RichText::new("Offline / Simulation").color(MUTED));
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center),|ui|{
                ui.add_space(6.0); ui.label(egui::RichText::new("NOT-AUS: Hardware erforderlich").color(Color32::from_rgb(180,30,30)).strong());
            });
        });
        egui::CentralPanel::default().show(ctx,|ui|{
            ui.columns(2,|cols|{
                cols[0].heading("G-Code Editor");
                cols[0].add(egui::TextEdit::multiline(&mut self.source).font(egui::TextStyle::Monospace).desired_rows(30).lock_focus(true));
                cols[1].horizontal(|ui|{ui.heading("Werkzeugbahn"); ui.label(egui::RichText::new(&self.status).color(ORANGE));});
                self.draw_path(&mut cols[1]);
            });
            ui.separator();
            ui.horizontal_wrapped(|ui|{
                metric(ui,"Segmente",self.report.segment_count.to_string());
                metric(ui,"Gesamtlänge",format!("{:.2} mm",self.report.total_length_mm));
                metric(ui,"Vorschubweg",format!("{:.2} mm",self.report.feed_length_mm));
                metric(ui,"Zeit grob",format!("{:.1} s",self.report.estimated_motion_seconds));
                metric(ui,"Endposition",format!("X {:.2} · Y {:.2} · Z {:.2}",self.report.final_position.x,self.report.final_position.y,self.report.final_position.z));
            });
            if !self.report.warnings.is_empty(){ ui.separator(); egui::CollapsingHeader::new(format!("Warnungen ({})",self.report.warnings.len())).default_open(true).show(ui,|ui|{for warning in &self.report.warnings{ui.colored_label(Color32::from_rgb(180,70,20),warning);}}); }
        });
    }
}

fn metric(ui:&mut egui::Ui,label:&str,value:String){
    egui::Frame::new().fill(ORANGE_SOFT).corner_radius(8.0).inner_margin(egui::Margin::symmetric(12,8)).show(ui,|ui|{ui.vertical(|ui|{ui.label(egui::RichText::new(label).small().color(MUTED));ui.label(egui::RichText::new(value).strong().color(TEXT));});});
}
