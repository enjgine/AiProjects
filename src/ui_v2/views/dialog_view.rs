// src/ui_v2/views/dialog_view.rs
//! Modal dialog view for confirmations and forms

use super::{View, BaseView};
use crate::ui_v2::core::{RenderContext, ComponentResult, InputEvent, ViewData, Layout};
use crate::ui_v2::components::{UIComponent, Button, Dropdown, TextInput};
use crate::core::events::PlayerCommand;
use macroquad::prelude::*;

/// Modal dialog for user interactions
pub struct DialogView {
    base: BaseView,
    dialog_type: DialogType,
    modal: bool,
    overlay_color: Color,
    content_text: String,
    buttons: Vec<DialogButton>,
    form_fields: Vec<FormField>,
    result_callback: Option<Box<dyn Fn(DialogResult) -> PlayerCommand>>,
}

#[derive(Debug, Clone)]
pub enum DialogType {
    Info,
    Warning,
    Error,
    Confirmation,
    Form,
    Custom,
}

#[derive(Debug, Clone)]
pub struct DialogButton {
    pub text: String,
    pub command: PlayerCommand,
    pub button_type: ButtonType,
}

#[derive(Debug, Clone)]
pub enum ButtonType {
    Primary,
    Secondary,
    Danger,
    Cancel,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub label: String,
    pub field_type: FieldType,
    pub value: String,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Text,
    Number,
    Dropdown(Vec<String>),
    Checkbox,
}

#[derive(Debug, Clone)]
pub enum DialogResult {
    Confirmed,
    Cancelled,
    FormSubmitted(Vec<(String, String)>),
}

impl DialogView {
    pub fn new(title: String, dialog_type: DialogType) -> Self {
        Self {
            base: BaseView::new(title),
            dialog_type,
            modal: true,
            overlay_color: Color::new(0.0, 0.0, 0.0, 0.5),
            content_text: String::new(),
            buttons: Vec::new(),
            form_fields: Vec::new(),
            result_callback: None,
        }
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content_text = content;
        self
    }

    pub fn with_buttons(mut self, buttons: Vec<DialogButton>) -> Self {
        self.buttons = buttons;
        self.rebuild_components();
        self
    }

    pub fn with_form_fields(mut self, fields: Vec<FormField>) -> Self {
        self.form_fields = fields;
        self.rebuild_components();
        self
    }

    pub fn with_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn(DialogResult) -> PlayerCommand + 'static,
    {
        self.result_callback = Some(Box::new(callback));
        self
    }

    pub fn set_modal(&mut self, modal: bool) {
        self.modal = modal;
    }

    pub fn center_on_screen(&mut self, screen_width: f32, screen_height: f32) {
        let dialog_width = 400.0;
        let dialog_height = self.calculate_height();
        
        self.base.layout = Layout::new(
            (screen_width - dialog_width) / 2.0,
            (screen_height - dialog_height) / 2.0,
            dialog_width,
            dialog_height
        );
        
        self.rebuild_components();
    }

    fn calculate_height(&self) -> f32 {
        let mut height = 60.0; // Title bar
        
        if !self.content_text.is_empty() {
            height += 80.0; // Content area
        }
        
        height += self.form_fields.len() as f32 * 40.0; // Form fields
        height += 60.0; // Button area
        height += self.base.layout.padding * 2.0;
        
        height
    }

    fn rebuild_components(&mut self) {
        self.base.components.clear();
        
        let content_area = self.base.get_content_area();
        let mut y_offset = content_area.y + 20.0;

        // Add form field components
        for field in &self.form_fields {
            match &field.field_type {
                FieldType::Text | FieldType::Number => {
                    // Would add TextInput component
                    y_offset += 40.0;
                }
                FieldType::Dropdown(options) => {
                    // Would add Dropdown component
                    y_offset += 40.0;
                }
                FieldType::Checkbox => {
                    // Would add Checkbox component
                    y_offset += 30.0;
                }
            }
        }

        // Add button components
        y_offset += 20.0;
        let button_width = 100.0;
        let button_spacing = 110.0;
        let total_button_width = self.buttons.len() as f32 * button_spacing - 10.0;
        let start_x = content_area.x + (content_area.w - total_button_width) / 2.0;

        for (i, dialog_button) in self.buttons.iter().enumerate() {
            let button = Button::new(dialog_button.text.clone())
                .with_click_command(dialog_button.command.clone())
                .with_layout(Layout::new(
                    start_x + i as f32 * button_spacing,
                    y_offset,
                    button_width,
                    35.0
                ));
            
            self.base.add_component(Box::new(button));
        }
    }

    fn render_content(&self, context: &RenderContext) -> ComponentResult {
        let content_area = self.base.get_content_area();
        let mut y_pos = content_area.y + 10.0;

        // Render content text
        if !self.content_text.is_empty() {
            let lines: Vec<&str> = self.content_text.lines().collect();
            for line in lines {
                draw_text(
                    line,
                    content_area.x + 10.0,
                    y_pos + 20.0,
                    context.font_size,
                    context.theme.text_color
                );
                y_pos += 25.0;
            }
            y_pos += 20.0;
        }

        // Render form fields
        for field in &self.form_fields {
            // Render label
            let label_text = if field.required {
                format!("{}*", field.label)
            } else {
                field.label.clone()
            };
            
            draw_text(
                &label_text,
                content_area.x + 10.0,
                y_pos + 15.0,
                context.font_size * 0.9,
                context.theme.text_color
            );

            // Render field based on type
            match &field.field_type {
                FieldType::Text | FieldType::Number => {
                    // Draw text input background
                    draw_rectangle(
                        content_area.x + 120.0,
                        y_pos,
                        200.0,
                        25.0,
                        context.theme.background_color
                    );
                    draw_rectangle_lines(
                        content_area.x + 120.0,
                        y_pos,
                        200.0,
                        25.0,
                        1.0,
                        context.theme.border_color
                    );
                    
                    // Draw current value
                    draw_text(
                        &field.value,
                        content_area.x + 125.0,
                        y_pos + 18.0,
                        context.font_size * 0.9,
                        context.theme.text_color
                    );
                }
                FieldType::Dropdown(_options) => {
                    // Draw dropdown (simplified representation)
                    draw_rectangle(
                        content_area.x + 120.0,
                        y_pos,
                        200.0,
                        25.0,
                        context.theme.background_color
                    );
                    draw_rectangle_lines(
                        content_area.x + 120.0,
                        y_pos,
                        200.0,
                        25.0,
                        1.0,
                        context.theme.border_color
                    );
                    
                    draw_text(
                        &field.value,
                        content_area.x + 125.0,
                        y_pos + 18.0,
                        context.font_size * 0.9,
                        context.theme.text_color
                    );
                    
                    draw_text(
                        "▼",
                        content_area.x + 300.0,
                        y_pos + 18.0,
                        context.font_size * 0.8,
                        context.theme.text_color
                    );
                }
                FieldType::Checkbox => {
                    // Draw checkbox
                    let checked = field.value == "true";
                    let checkbox_color = if checked {
                        context.theme.primary_color
                    } else {
                        context.theme.background_color
                    };
                    
                    draw_rectangle(
                        content_area.x + 120.0,
                        y_pos + 2.0,
                        20.0,
                        20.0,
                        checkbox_color
                    );
                    draw_rectangle_lines(
                        content_area.x + 120.0,
                        y_pos + 2.0,
                        20.0,
                        20.0,
                        1.0,
                        context.theme.border_color
                    );
                    
                    if checked {
                        draw_text(
                            "✓",
                            content_area.x + 125.0,
                            y_pos + 18.0,
                            context.font_size * 0.9,
                            context.theme.text_color
                        );
                    }
                }
            }

            y_pos += 40.0;
        }

        Ok(None)
    }

    fn render_modal_overlay(&self, context: &RenderContext) -> ComponentResult {
        if self.modal {
            draw_rectangle(
                0.0, 0.0,
                context.screen_width,
                context.screen_height,
                self.overlay_color
            );
        }
        Ok(None)
    }
}

impl View for DialogView {
    fn render(&mut self, context: &RenderContext) -> ComponentResult {
        // Render modal overlay first
        self.render_modal_overlay(context)?;
        
        // Render dialog base
        self.base.render_base(context)?;
        
        // Render dialog content
        self.render_content(context)?;
        
        // Render components (buttons)
        self.base.render_components(context)?;

        Ok(None)
    }

    fn handle_input(&mut self, input: &InputEvent) -> ComponentResult {
        // For modal dialogs, block input outside the dialog
        if self.modal {
            if let InputEvent::MouseClick { x, y, .. } = input {
                let mouse_pos = Vec2::new(*x, *y);
                if !self.base.contains_point(mouse_pos) {
                    return Ok(None); // Block the input
                }
            }
        }

        // Handle escape key for cancellation
        if let InputEvent::KeyPress { key } = input {
            if *key == KeyCode::Escape {
                if let Some(callback) = &self.result_callback {
                    return Ok(Some(callback(DialogResult::Cancelled)));
                }
            }
        }

        self.base.handle_input_base(input)
    }

    fn update(&mut self, delta_time: f32) -> ComponentResult {
        self.base.update_components(delta_time)
    }

    fn update_data(&mut self, _data: ViewData) -> ComponentResult {
        Ok(None)
    }

    fn is_visible(&self) -> bool {
        self.base.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.base.visible = visible;
    }

    fn refresh(&mut self) -> ComponentResult {
        self.rebuild_components();
        Ok(None)
    }

    fn get_view_type(&self) -> &'static str {
        "DialogView"
    }
}

// Helper functions for creating common dialogs
impl DialogView {
    pub fn confirmation(title: String, message: String, on_confirm: PlayerCommand) -> Self {
        Self::new(title, DialogType::Confirmation)
            .with_content(message)
            .with_buttons(vec![
                DialogButton {
                    text: "Yes".to_string(),
                    command: on_confirm,
                    button_type: ButtonType::Primary,
                },
                DialogButton {
                    text: "Cancel".to_string(),
                    command: PlayerCommand::ClosePlanetPanel, // Placeholder
                    button_type: ButtonType::Cancel,
                },
            ])
    }

    pub fn info(title: String, message: String) -> Self {
        Self::new(title, DialogType::Info)
            .with_content(message)
            .with_buttons(vec![
                DialogButton {
                    text: "OK".to_string(),
                    command: PlayerCommand::ClosePlanetPanel, // Placeholder
                    button_type: ButtonType::Primary,
                },
            ])
    }
}