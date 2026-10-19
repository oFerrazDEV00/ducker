//! Sistema Duckive: identidade do pato, gamificação, inventário de cosméticos e conquistas.
//! Salva em `<config_dir>/duckive.json`.

use std::collections::HashSet;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{DuckerError, Result};

pub const DUCKIVE_FILE: &str = "duckive.json";

/// Categorias de itens cosméticos do Duckive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CosmeticCategory {
    Head,
    Outfit,
    Accessory,
    Effect,
}

impl CosmeticCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Head => "Cabeça",
            Self::Outfit => "Roupa",
            Self::Accessory => "Acessório",
            Self::Effect => "Efeito",
        }
    }
}

/// Raridade do cosmético.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Common => "Comum",
            Self::Rare => "Raro",
            Self::Epic => "Épico",
            Self::Legendary => "Lendário",
        }
    }
}

/// Critério de desbloqueio de um item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum UnlockRequirement {
    DefaultUnlocked,
    FilesSent(u64),
    Achievement(String),
    Special(String),
}

/// Definição imutável de um item cosmético no catálogo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CosmeticItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CosmeticCategory,
    pub rarity: Rarity,
    pub requirement: UnlockRequirement,
    pub icon_emoji: String,
    pub asset_id: String,
}

/// Itens equipados nos diferentes slots.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EquippedItems {
    #[serde(default)]
    pub head: Option<String>,
    #[serde(default)]
    pub outfit: Option<String>,
    #[serde(default)]
    pub accessory: Option<String>,
    #[serde(default)]
    pub effect: Option<String>,
}

/// Definição de uma conquista.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AchievementDefinition {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_emoji: String,
}

/// Estado individual de uma conquista do usuário.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AchievementState {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon_emoji: String,
    pub unlocked: bool,
    #[serde(default)]
    pub unlocked_at_ms: Option<u64>,
}

/// Estado persistente do Duckive do usuário.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DuckiveProfile {
    #[serde(default = "default_duckive_name")]
    pub name: String,
    #[serde(default)]
    pub files_sent_count: u64,
    #[serde(default)]
    pub files_received_count: u64,
    #[serde(default)]
    pub total_bytes_sent: u64,
    #[serde(default)]
    pub total_bytes_received: u64,
    #[serde(default)]
    pub transfers_count: u64,
    #[serde(default)]
    pub equipped: EquippedItems,
    #[serde(default)]
    pub unlocked_items: Vec<String>,
    #[serde(default)]
    pub achievements: Vec<AchievementState>,
    #[serde(default)]
    pub created_at_ms: u64,
    #[serde(default)]
    pub updated_at_ms: u64,
}

fn default_duckive_name() -> String {
    "Duckive".to_string()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Resumo de novas conquistas e itens desbloqueados após uma transferência.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UnlockSummary {
    pub newly_unlocked_items: Vec<CosmeticItem>,
    pub newly_unlocked_achievements: Vec<AchievementState>,
    pub total_files_sent: u64,
    pub level_up: bool,
    pub current_level: u32,
}

/// Status de um item cosmético na visão do inventário.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CosmeticItemStatus {
    pub item: CosmeticItem,
    pub unlocked: bool,
    pub equipped: bool,
    pub progress_current: u64,
    pub progress_required: u64,
    pub progress_pct: u32,
}

/// Informações de nível e progressão.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfo {
    pub level: u32,
    pub title: String,
    pub current_files: u64,
    pub current_level_min_files: u64,
    pub next_level_files: Option<u64>,
    pub progress_pct: f32,
    pub next_reward_item: Option<CosmeticItem>,
}

/// DTO completo retornado para a UI e CLI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DuckiveView {
    pub name: String,
    pub level: LevelInfo,
    pub files_sent_count: u64,
    pub files_received_count: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub transfers_count: u64,
    pub equipped: EquippedItems,
    pub items: Vec<CosmeticItemStatus>,
    pub achievements: Vec<AchievementState>,
}

/// Permissões granulares da conta de administrador (Faelindo).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdminPermissions {
    pub can_view_stats: bool,
    pub can_view_devices: bool,
    pub can_manage_cosmetics: bool,
    pub can_test_unlocks: bool,
    pub can_view_logs: bool,
    pub can_run_diagnostics: bool,
    pub can_dev_config: bool,
}

impl Default for AdminPermissions {
    fn default() -> Self {
        Self {
            can_view_stats: true,
            can_view_devices: true,
            can_manage_cosmetics: true,
            can_test_unlocks: true,
            can_view_logs: true,
            can_run_diagnostics: true,
            can_dev_config: true,
        }
    }
}

/// Sessão administrativa autenticada no backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdminSession {
    pub is_admin: bool,
    pub admin_name: String,
    pub role: String,
    pub is_dev_mode: bool,
    pub permissions: AdminPermissions,
}

impl Default for AdminSession {
    fn default() -> Self {
        Self {
            is_admin: false,
            admin_name: String::new(),
            role: "Usuário Comum".into(),
            is_dev_mode: false,
            permissions: AdminPermissions {
                can_view_stats: false,
                can_view_devices: false,
                can_manage_cosmetics: false,
                can_test_unlocks: false,
                can_view_logs: false,
                can_run_diagnostics: false,
                can_dev_config: false,
            },
        }
    }
}

impl DuckiveProfile {
    /// Desbloqueia manualmente um item (usado exclusivamente pelo Admin para testes).
    pub fn admin_unlock_item(&mut self, item_id: &str) -> Option<CosmeticItem> {
        let catalog = get_cosmetics_catalog();
        if let Some(item) = catalog.into_iter().find(|it| it.id == item_id) {
            if !self.unlocked_items.contains(&item.id) {
                self.unlocked_items.push(item.id.clone());
                self.updated_at_ms = now_ms();
                return Some(item);
            }
        }
        None
    }

    /// Inicializa um perfil Duckive novo com itens e conquistas padrão.
    pub fn new(name: impl Into<String>) -> Self {
        let ts = now_ms();
        let mut profile = Self {
            name: name.into(),
            files_sent_count: 0,
            files_received_count: 0,
            total_bytes_sent: 0,
            total_bytes_received: 0,
            transfers_count: 0,
            equipped: EquippedItems::default(),
            unlocked_items: Vec::new(),
            achievements: Vec::new(),
            created_at_ms: ts,
            updated_at_ms: ts,
        };
        profile.reconcile_catalog();
        profile
    }

    /// Garante que itens padrão e definições de conquistas existam no perfil, sem perder progresso existente.
    pub fn reconcile_catalog(&mut self) {
        // 1. Reconcilia catálogo de conquistas
        let ach_catalog = get_achievements_catalog();
        let existing_map: HashSet<String> =
            self.achievements.iter().map(|a| a.id.clone()).collect();
        for ach in ach_catalog {
            if !existing_map.contains(&ach.id) {
                self.achievements.push(AchievementState {
                    id: ach.id,
                    title: ach.title,
                    description: ach.description,
                    icon_emoji: ach.icon_emoji,
                    unlocked: false,
                    unlocked_at_ms: None,
                });
            }
        }

        // Valida conquistas que dependem puramente de contadores persistidos
        if self.transfers_count >= 1 {
            self.unlock_achievement_internal("first_flight");
        }
        if self.files_sent_count >= 100 {
            self.unlock_achievement_internal("centurion");
        }
        if self.files_sent_count >= 50000 {
            self.unlock_achievement_internal("whatsapp_green_legend");
        }
        let has_custom = self.equipped.head.is_some()
            || self.equipped.outfit.is_some()
            || self.equipped.accessory.is_some()
            || self.equipped.effect.is_some();
        if has_custom {
            self.unlock_achievement_internal("stylist");
        }

        // 2. Reconcilia catálogo de cosméticos
        let catalog = get_cosmetics_catalog();
        let mut unlocked_set: HashSet<String> = self.unlocked_items.iter().cloned().collect();

        // Desbloqueia itens padrão, por contagem de arquivos ou por conquista concluída
        for item in &catalog {
            match &item.requirement {
                UnlockRequirement::DefaultUnlocked => {
                    unlocked_set.insert(item.id.clone());
                }
                UnlockRequirement::FilesSent(req) => {
                    if self.files_sent_count >= *req {
                        unlocked_set.insert(item.id.clone());
                    }
                }
                UnlockRequirement::Achievement(ach_id) => {
                    if self
                        .achievements
                        .iter()
                        .any(|a| a.id == *ach_id && a.unlocked)
                    {
                        unlocked_set.insert(item.id.clone());
                    }
                }
                _ => {}
            }
        }

        // Preserva a ordem do catálogo e retém itens já conquistados anteriormente
        let mut ordered: Vec<String> = catalog
            .iter()
            .filter(|it| unlocked_set.contains(&it.id))
            .map(|it| it.id.clone())
            .collect();

        // Preserva quaisquer itens desbloqueados que não constem do catálogo base
        for id in &unlocked_set {
            if !ordered.contains(id) {
                ordered.push(id.clone());
            }
        }

        self.unlocked_items = ordered;
    }

    /// Carrega o perfil de `<dir>/duckive.json`.
    pub fn load(dir: &Path) -> Result<Option<Self>> {
        let path = dir.join(DUCKIVE_FILE);
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        let clean = content.strip_prefix('\u{feff}').unwrap_or(&content);
        let mut profile: Self = serde_json::from_str(clean)?;
        profile.reconcile_catalog();
        Ok(Some(profile))
    }

    /// Salva o perfil em `<dir>/duckive.json`.
    pub fn save(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(DUCKIVE_FILE);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Carrega do diretório ou cria um perfil novo com `default_name`.
    pub fn load_or_create(dir: &Path, default_name: &str) -> Result<Self> {
        if let Some(profile) = Self::load(dir)? {
            return Ok(profile);
        }
        let profile = Self::new(default_name);
        let _ = profile.save(dir);
        Ok(profile)
    }

    /// Equipa um item desbloqueado no slot correspondente.
    pub fn equip_item(&mut self, item_id: &str) -> Result<()> {
        let catalog = get_cosmetics_catalog();
        let item = catalog
            .into_iter()
            .find(|it| it.id == item_id)
            .ok_or_else(|| {
                DuckerError::Custom(format!("Item cosmético '{}' não existe", item_id))
            })?;

        if !self.unlocked_items.contains(&item.id) {
            return Err(DuckerError::Custom(format!(
                "Item '{}' ainda está bloqueado",
                item.name
            )));
        }

        match item.category {
            CosmeticCategory::Head => {
                if self.equipped.head.as_deref() == Some(&item.id) {
                    self.equipped.head = None;
                } else {
                    self.equipped.head = Some(item.id.clone());
                }
            }
            CosmeticCategory::Outfit => {
                if self.equipped.outfit.as_deref() == Some(&item.id) {
                    self.equipped.outfit = None;
                } else {
                    self.equipped.outfit = Some(item.id.clone());
                }
            }
            CosmeticCategory::Accessory => {
                if self.equipped.accessory.as_deref() == Some(&item.id) {
                    self.equipped.accessory = None;
                } else {
                    self.equipped.accessory = Some(item.id.clone());
                }
            }
            CosmeticCategory::Effect => {
                if self.equipped.effect.as_deref() == Some(&item.id) {
                    self.equipped.effect = None;
                } else {
                    self.equipped.effect = Some(item.id.clone());
                }
            }
        }

        // Verifica conquista "Pato Estiloso" (equipar qualquer item que não seja o padrão)
        let has_custom = self.equipped.head.is_some()
            || self.equipped.outfit.is_some()
            || self.equipped.accessory.is_some()
            || self.equipped.effect.is_some();
        if has_custom {
            self.unlock_achievement_internal("stylist");
        }

        self.updated_at_ms = now_ms();
        Ok(())
    }

    /// Desequipa um slot específico.
    pub fn unequip_slot(&mut self, category: CosmeticCategory) {
        match category {
            CosmeticCategory::Head => self.equipped.head = None,
            CosmeticCategory::Outfit => self.equipped.outfit = None,
            CosmeticCategory::Accessory => self.equipped.accessory = None,
            CosmeticCategory::Effect => self.equipped.effect = None,
        }
        self.updated_at_ms = now_ms();
    }

    /// Registra arquivos recebidos.
    pub fn record_files_received(&mut self, count: u64, bytes: u64) {
        self.files_received_count += count;
        self.total_bytes_received += bytes;
        self.updated_at_ms = now_ms();
    }

    /// Registra progresso de transferência e computa novos desbloqueios.
    pub fn record_transfer(
        &mut self,
        files_sent: u64,
        bytes: u64,
        is_cross_platform: bool,
        speed_mbps: f64,
        is_text: bool,
    ) -> UnlockSummary {
        let prev_level = self.calculate_level_info().level;
        let mut newly_unlocked_items = Vec::new();
        let mut newly_unlocked_achievements = Vec::new();

        if files_sent == 0 && !is_text && bytes == 0 {
            return UnlockSummary {
                newly_unlocked_items: Vec::new(),
                newly_unlocked_achievements: Vec::new(),
                total_files_sent: self.files_sent_count,
                level_up: false,
                current_level: self.calculate_level_info().level,
            };
        }

        self.files_sent_count += files_sent;
        self.total_bytes_sent += bytes;
        self.transfers_count += 1;
        self.updated_at_ms = now_ms();

        // 1. Avalia Conquistas
        // "Primeiro Voo": primeira transferência concluída
        if self.transfers_count >= 1 {
            if let Some(ach) = self.unlock_achievement_internal("first_flight") {
                newly_unlocked_achievements.push(ach);
            }
        }

        // "Pato Multiplataforma": transferência entre plataformas diferentes
        if is_cross_platform {
            if let Some(ach) = self.unlock_achievement_internal("cross_platform") {
                newly_unlocked_achievements.push(ach);
            }
        }

        // "Pato Turbo": velocidade > 5.0 MB/s
        if speed_mbps >= 5.0 {
            if let Some(ach) = self.unlock_achievement_internal("turbo_duck") {
                newly_unlocked_achievements.push(ach);
            }
        }

        // "Pato Verde do WhatsApp": envio de mensagem rápida de texto
        if is_text {
            if let Some(ach) = self.unlock_achievement_internal("whatsapp_green") {
                newly_unlocked_achievements.push(ach);
            }
        }

        // "Centurião dos Arquivos": 100 arquivos enviados
        if self.files_sent_count >= 100 {
            if let Some(ach) = self.unlock_achievement_internal("centurion") {
                newly_unlocked_achievements.push(ach);
            }
        }

        // "Mestre Esmeralda WhatsApp": 50.000 arquivos enviados
        if self.files_sent_count >= 50000 {
            if let Some(ach) = self.unlock_achievement_internal("whatsapp_green_legend") {
                newly_unlocked_achievements.push(ach);
            }
        }

        // 2. Avalia Cosméticos desbloqueáveis por arquivos ou por conquistas
        let catalog = get_cosmetics_catalog();
        let currently_unlocked: HashSet<String> = self.unlocked_items.iter().cloned().collect();

        for item in catalog {
            if currently_unlocked.contains(&item.id) {
                continue;
            }

            let can_unlock = match &item.requirement {
                UnlockRequirement::DefaultUnlocked => true,
                UnlockRequirement::FilesSent(req) => self.files_sent_count >= *req,
                UnlockRequirement::Achievement(ach_id) => self
                    .achievements
                    .iter()
                    .any(|a| a.id == *ach_id && a.unlocked),
                UnlockRequirement::Special(_) => false,
            };

            if can_unlock {
                self.unlocked_items.push(item.id.clone());
                newly_unlocked_items.push(item);
            }
        }

        let curr_level = self.calculate_level_info().level;

        UnlockSummary {
            newly_unlocked_items,
            newly_unlocked_achievements,
            total_files_sent: self.files_sent_count,
            level_up: curr_level > prev_level,
            current_level: curr_level,
        }
    }

    fn unlock_achievement_internal(&mut self, ach_id: &str) -> Option<AchievementState> {
        let ts = now_ms();
        for ach in &mut self.achievements {
            if ach.id == ach_id && !ach.unlocked {
                ach.unlocked = true;
                ach.unlocked_at_ms = Some(ts);
                return Some(ach.clone());
            }
        }
        None
    }

    /// Calcula informações completas sobre o nível atual do Duckive.
    pub fn calculate_level_info(&self) -> LevelInfo {
        let milestones: &[(u32, &str, u64)] = &[
            (1, "Pato Recruta", 0),
            (2, "Pato Estiloso", 10),
            (3, "Pato Descolado", 25),
            (4, "Pato Aconchegante", 50),
            (5, "Pato Executivo", 100),
            (6, "Pato Programador", 250),
            (7, "Pato Astronauta", 500),
            (8, "Pato Lendário", 1000),
            (9, "Pato Cósmico Supremo", 5000),
            (10, "Mestre Esmeralda WhatsApp", 50000),
        ];

        let files = self.files_sent_count;
        let mut current_idx = 0;

        for (idx, &(_, _, req)) in milestones.iter().enumerate() {
            if files >= req {
                current_idx = idx;
            } else {
                break;
            }
        }

        let (level, title, min_files) = milestones[current_idx];
        let next_milestone = milestones.get(current_idx + 1);

        let (next_files, progress_pct) = match next_milestone {
            Some(&(_, _, next_req)) => {
                let diff = next_req.saturating_sub(min_files);
                let current_offset = files.saturating_sub(min_files);
                let pct = if diff == 0 {
                    100.0
                } else {
                    ((current_offset as f32 / diff as f32) * 100.0).clamp(0.0, 100.0)
                };
                (Some(next_req), pct)
            }
            None => (None, 100.0),
        };

        // Próximo item de recompensa
        let catalog = get_cosmetics_catalog();
        let next_reward_item = catalog.into_iter().find(|it| {
            if let UnlockRequirement::FilesSent(req) = it.requirement {
                req > files
            } else {
                false
            }
        });

        LevelInfo {
            level,
            title: title.to_string(),
            current_files: files,
            current_level_min_files: min_files,
            next_level_files: next_files,
            progress_pct,
            next_reward_item,
        }
    }

    /// Gera uma visão serializável para UI e CLI com o status de cada cosmético.
    pub fn to_view(&self) -> DuckiveView {
        let catalog = get_cosmetics_catalog();
        let unlocked_set: HashSet<String> = self.unlocked_items.iter().cloned().collect();

        let items: Vec<CosmeticItemStatus> = catalog
            .into_iter()
            .map(|item| {
                let unlocked = unlocked_set.contains(&item.id);
                let equipped = match item.category {
                    CosmeticCategory::Head => self.equipped.head.as_deref() == Some(&item.id),
                    CosmeticCategory::Outfit => self.equipped.outfit.as_deref() == Some(&item.id),
                    CosmeticCategory::Accessory => {
                        self.equipped.accessory.as_deref() == Some(&item.id)
                    }
                    CosmeticCategory::Effect => self.equipped.effect.as_deref() == Some(&item.id),
                };

                let (current, req) = match item.requirement {
                    UnlockRequirement::DefaultUnlocked => (1, 1),
                    UnlockRequirement::FilesSent(r) => (self.files_sent_count.min(r), r),
                    UnlockRequirement::Achievement(ref ach_id) => {
                        let is_ach_done = self
                            .achievements
                            .iter()
                            .any(|a| a.id == *ach_id && a.unlocked);
                        (if is_ach_done { 1 } else { 0 }, 1)
                    }
                    UnlockRequirement::Special(_) => (if unlocked { 1 } else { 0 }, 1),
                };

                let pct = if req == 0 {
                    100
                } else {
                    ((current as f64 / req as f64) * 100.0).clamp(0.0, 100.0) as u32
                };

                CosmeticItemStatus {
                    item,
                    unlocked,
                    equipped,
                    progress_current: current,
                    progress_required: req,
                    progress_pct: pct,
                }
            })
            .collect();

        DuckiveView {
            name: self.name.clone(),
            level: self.calculate_level_info(),
            files_sent_count: self.files_sent_count,
            files_received_count: self.files_received_count,
            total_bytes_sent: self.total_bytes_sent,
            total_bytes_received: self.total_bytes_received,
            transfers_count: self.transfers_count,
            equipped: self.equipped.clone(),
            items,
            achievements: self.achievements.clone(),
        }
    }
}

/// Catálogo imutável e extensível de cosméticos do Duckive.
pub fn get_cosmetics_catalog() -> Vec<CosmeticItem> {
    vec![
        // 0 Arquivos: Padrão
        CosmeticItem {
            id: "default_duck".into(),
            name: "Duckive Clássico".into(),
            description: "O pato original do Ducker em sua forma pura e amarela.".into(),
            category: CosmeticCategory::Effect,
            rarity: Rarity::Common,
            requirement: UnlockRequirement::DefaultUnlocked,
            icon_emoji: "🦆".into(),
            asset_id: "default_duck".into(),
        },
        // 10 Arquivos: Boné
        CosmeticItem {
            id: "cap_ducker".into(),
            name: "Boné Ducker".into(),
            description: "Boné esportivo com aba curva para manter o foco na transferência.".into(),
            category: CosmeticCategory::Head,
            rarity: Rarity::Common,
            requirement: UnlockRequirement::FilesSent(10),
            icon_emoji: "🧢".into(),
            asset_id: "cap_ducker".into(),
        },
        // 25 Arquivos: Óculos
        CosmeticItem {
            id: "sunglasses_cool".into(),
            name: "Óculos Escuros Maneiros".into(),
            description: "Estilo puro e proteção contra o brilho dos pacotes de rede.".into(),
            category: CosmeticCategory::Accessory,
            rarity: Rarity::Rare,
            requirement: UnlockRequirement::FilesSent(25),
            icon_emoji: "🕶️".into(),
            asset_id: "sunglasses_cool".into(),
        },
        // 50 Arquivos: Moletom
        CosmeticItem {
            id: "hoodie_ducker".into(),
            name: "Moletom Amarelo Ducker".into(),
            description: "Conforto premium para transferências em noites frias de código.".into(),
            category: CosmeticCategory::Outfit,
            rarity: Rarity::Rare,
            requirement: UnlockRequirement::FilesSent(50),
            icon_emoji: "🧥".into(),
            asset_id: "hoodie_ducker".into(),
        },
        // 100 Arquivos: Terno
        CosmeticItem {
            id: "business_suit".into(),
            name: "Terno Executivo".into(),
            description: "Para quando suas transferências P2P exigem seriedade corporativa.".into(),
            category: CosmeticCategory::Outfit,
            rarity: Rarity::Epic,
            requirement: UnlockRequirement::FilesSent(100),
            icon_emoji: "👔".into(),
            asset_id: "business_suit".into(),
        },
        // 250 Arquivos: Roupa de Programador
        CosmeticItem {
            id: "dev_coder".into(),
            name: "Roupa de Programador".into(),
            description: "Camiseta preta estampada com 'while(alive) { quac(); }' e fones de ouvido.".into(),
            category: CosmeticCategory::Outfit,
            rarity: Rarity::Epic,
            requirement: UnlockRequirement::FilesSent(250),
            icon_emoji: "💻".into(),
            asset_id: "dev_coder".into(),
        },
        // 500 Arquivos: Astronauta (Roupa)
        CosmeticItem {
            id: "astronaut_suit".into(),
            name: "Traje Espacial Ducker".into(),
            description: "Blindagem pressurizada para enviar pacotes até a órbita lunar.".into(),
            category: CosmeticCategory::Outfit,
            rarity: Rarity::Epic,
            requirement: UnlockRequirement::FilesSent(500),
            icon_emoji: "🚀".into(),
            asset_id: "astronaut_suit".into(),
        },
        // 500 Arquivos: Astronauta (Capacete)
        CosmeticItem {
            id: "astronaut_helmet".into(),
            name: "Capacete de Astronauta".into(),
            description: "Visor espelhado dourado resistente ao vácuo intergaláctico.".into(),
            category: CosmeticCategory::Head,
            rarity: Rarity::Epic,
            requirement: UnlockRequirement::FilesSent(500),
            icon_emoji: "👨‍🚀".into(),
            asset_id: "astronaut_helmet".into(),
        },
        // 1.000 Arquivos: Coroa Real Lendária
        CosmeticItem {
            id: "legendary_crown".into(),
            name: "Coroa Real Dourada".into(),
            description: "Feita em ouro maciço Quac para o soberano absoluto da rede local.".into(),
            category: CosmeticCategory::Head,
            rarity: Rarity::Legendary,
            requirement: UnlockRequirement::FilesSent(1000),
            icon_emoji: "👑".into(),
            asset_id: "legendary_crown".into(),
        },
        // 1.000 Arquivos: Capa Lendária
        CosmeticItem {
            id: "legendary_cape".into(),
            name: "Manto Lendário Real".into(),
            description: "Capa aveludada vermelha e dourada digna de lendas da computação.".into(),
            category: CosmeticCategory::Outfit,
            rarity: Rarity::Legendary,
            requirement: UnlockRequirement::FilesSent(1000),
            icon_emoji: "✨".into(),
            asset_id: "legendary_cape".into(),
        },
        // 5.000 Arquivos: Recompensa Especial / Cósmica
        CosmeticItem {
            id: "cosmic_duck".into(),
            name: "Aura Cósmica Suprema".into(),
            description: "Emanação de energia astral reservada aos maiores mestres do Ducker.".into(),
            category: CosmeticCategory::Effect,
            rarity: Rarity::Legendary,
            requirement: UnlockRequirement::FilesSent(5000),
            icon_emoji: "🌌".into(),
            asset_id: "cosmic_duck".into(),
        },
        // 50.000 Arquivos: Recompensa Lendária Suprema — PATO VERDE DO WHATSAPP
        CosmeticItem {
            id: "whatsapp_green_skin".into(),
            name: "Pato Verde do WhatsApp".into(),
            description: "O lendário Pato Verde do WhatsApp. Concedido exclusivamente aos mestres que alcançaram a marca histórica de 50.000 arquivos enviados pelo Ducker.".into(),
            category: CosmeticCategory::Effect,
            rarity: Rarity::Legendary,
            requirement: UnlockRequirement::FilesSent(50000),
            icon_emoji: "🟢".into(),
            asset_id: "whatsapp_green_skin".into(),
        },
        // Cosméticos por Conquistas
        CosmeticItem {
            id: "messenger_badge".into(),
            name: "Pin Mensageiro Veloz".into(),
            description: "Insígnia concedida pela primeira mensagem de texto enviada pelo Ducker.".into(),
            category: CosmeticCategory::Accessory,
            rarity: Rarity::Rare,
            requirement: UnlockRequirement::Achievement("whatsapp_green".into()),
            icon_emoji: "💬".into(),
            asset_id: "messenger_badge".into(),
        },
        CosmeticItem {
            id: "turbo_goggles".into(),
            name: "Óculos Turbo Aerodinâmicos".into(),
            description: "Desenhados para reduzir o atrito em transferências acima de 5 MB/s.".into(),
            category: CosmeticCategory::Accessory,
            rarity: Rarity::Rare,
            requirement: UnlockRequirement::Achievement("turbo_duck".into()),
            icon_emoji: "⚡".into(),
            asset_id: "turbo_goggles".into(),
        },
        CosmeticItem {
            id: "globe_trotter_badge".into(),
            name: "Pin Pato Multiplataforma".into(),
            description: "Insígnia concedida por cruzar as fronteiras entre diferentes SOs.".into(),
            category: CosmeticCategory::Accessory,
            rarity: Rarity::Rare,
            requirement: UnlockRequirement::Achievement("cross_platform".into()),
            icon_emoji: "🌐".into(),
            asset_id: "globe_trotter_badge".into(),
        },
    ]
}

/// Catálogo imutável e extensível de conquistas do Ducker.
pub fn get_achievements_catalog() -> Vec<AchievementDefinition> {
    vec![
        AchievementDefinition {
            id: "first_flight".into(),
            title: "Primeiro Voo".into(),
            description: "Completou a primeira transferência de arquivos com sucesso.".into(),
            icon_emoji: "🦆".into(),
        },
        AchievementDefinition {
            id: "cross_platform".into(),
            title: "Pato Multiplataforma".into(),
            description: "Transferiu arquivos entre sistemas operacionais diferentes.".into(),
            icon_emoji: "🌐".into(),
        },
        AchievementDefinition {
            id: "turbo_duck".into(),
            title: "Pato Turbo".into(),
            description: "Atingiu velocidade superior a 5 MB/s em uma transferência.".into(),
            icon_emoji: "⚡".into(),
        },
        AchievementDefinition {
            id: "whatsapp_green".into(),
            title: "Pato Verde do WhatsApp".into(),
            description: "Enviou uma mensagem de texto rápida pelo Ducker.".into(),
            icon_emoji: "💬".into(),
        },
        AchievementDefinition {
            id: "centurion".into(),
            title: "Centurião dos Arquivos".into(),
            description: "Alcançou a marca histórica de 100 arquivos enviados.".into(),
            icon_emoji: "💯".into(),
        },
        AchievementDefinition {
            id: "whatsapp_green_legend".into(),
            title: "Mestre Esmeralda WhatsApp".into(),
            description: "Alcançou a marca épica de 50.000 arquivos enviados e libertou o lendário Pato Verde do WhatsApp.".into(),
            icon_emoji: "🟢".into(),
        },
        AchievementDefinition {
            id: "stylist".into(),
            title: "Pato Estiloso".into(),
            description: "Personalizou e equipou seu primeiro cosmético no Duckive.".into(),
            icon_emoji: "🎩".into(),
        },

    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_profile_starts_at_level_1_with_default_duck() {
        let profile = DuckiveProfile::new("Fael-Duck");
        assert_eq!(profile.name, "Fael-Duck");
        assert_eq!(profile.files_sent_count, 0);
        assert!(profile.unlocked_items.contains(&"default_duck".to_string()));

        let view = profile.to_view();
        assert_eq!(view.level.level, 1);
        assert_eq!(view.level.title, "Pato Recruta");
        assert_eq!(view.level.next_level_files, Some(10));
    }

    #[test]
    fn file_progression_unlocks_items_in_cascade() {
        let mut profile = DuckiveProfile::new("Test");
        assert!(!profile.unlocked_items.contains(&"cap_ducker".to_string()));

        // Envia 5 arquivos: ainda não desbloqueia o boné (precisa de 10)
        let summary1 = profile.record_transfer(5, 1024, false, 2.0, false);
        assert_eq!(summary1.newly_unlocked_items.len(), 0);
        assert_eq!(profile.calculate_level_info().level, 1);

        // Envia mais 5 (total 10): desbloqueia o boné!
        let summary2 = profile.record_transfer(5, 1024, false, 2.0, false);
        assert!(summary2
            .newly_unlocked_items
            .iter()
            .any(|i| i.id == "cap_ducker"));
        assert_eq!(profile.calculate_level_info().level, 2);
        assert_eq!(profile.calculate_level_info().title, "Pato Estiloso");

        // Envia mais 15 (total 25): desbloqueia os óculos
        let summary3 = profile.record_transfer(15, 2048, false, 2.0, false);
        assert!(summary3
            .newly_unlocked_items
            .iter()
            .any(|i| i.id == "sunglasses_cool"));
        assert_eq!(profile.calculate_level_info().level, 3);
    }

    #[test]
    fn equipping_and_unequipping_items() {
        let mut profile = DuckiveProfile::new("Test");
        // Tentar equipar item bloqueado dá erro
        assert!(profile.equip_item("cap_ducker").is_err());

        // Atinge 10 arquivos
        profile.record_transfer(10, 1024, false, 1.0, false);
        assert!(profile.unlocked_items.contains(&"cap_ducker".to_string()));

        // Equipa boné
        profile.equip_item("cap_ducker").unwrap();
        assert_eq!(profile.equipped.head.as_deref(), Some("cap_ducker"));

        // Desequipa slot
        profile.unequip_slot(CosmeticCategory::Head);
        assert_eq!(profile.equipped.head, None);
    }

    #[test]
    fn unlocks_achievements_appropriately() {
        let mut profile = DuckiveProfile::new("Test");

        // 1ª transferência desbloqueia "Primeiro Voo"
        let res = profile.record_transfer(1, 500, true, 8.5, true);
        assert!(res
            .newly_unlocked_achievements
            .iter()
            .any(|a| a.id == "first_flight"));
        assert!(res
            .newly_unlocked_achievements
            .iter()
            .any(|a| a.id == "cross_platform"));
        assert!(res
            .newly_unlocked_achievements
            .iter()
            .any(|a| a.id == "turbo_duck"));
        assert!(res
            .newly_unlocked_achievements
            .iter()
            .any(|a| a.id == "whatsapp_green"));

        // O item associado à conquista do whatsapp green deve estar liberado
        assert!(profile
            .unlocked_items
            .contains(&"messenger_badge".to_string()));
    }

    #[test]
    fn fifty_thousand_files_unlocks_legendary_whatsapp_green_skin() {
        let mut profile = DuckiveProfile::new("Test");
        assert!(!profile
            .unlocked_items
            .contains(&"whatsapp_green_skin".to_string()));

        // Envia 49.999 arquivos: ainda não desbloqueia
        let summary1 = profile.record_transfer(49_999, 100_000_000, false, 5.0, false);
        assert!(!summary1
            .newly_unlocked_items
            .iter()
            .any(|i| i.id == "whatsapp_green_skin"));
        assert!(!profile
            .unlocked_items
            .contains(&"whatsapp_green_skin".to_string()));

        // Envia o 50.000º arquivo: desbloqueia permanentemente o PATO VERDE DO WHATSAPP!
        let summary2 = profile.record_transfer(1, 1024, false, 5.0, false);
        assert!(summary2
            .newly_unlocked_items
            .iter()
            .any(|i| i.id == "whatsapp_green_skin"));
        assert!(profile
            .unlocked_items
            .contains(&"whatsapp_green_skin".to_string()));

        // O item é lendário
        let item = summary2
            .newly_unlocked_items
            .iter()
            .find(|i| i.id == "whatsapp_green_skin")
            .unwrap();
        assert_eq!(item.rarity, Rarity::Legendary);

        // Pode ser equipado
        profile.equip_item("whatsapp_green_skin").unwrap();
        assert_eq!(
            profile.equipped.effect.as_deref(),
            Some("whatsapp_green_skin")
        );

        // Nível 10 atingido
        let lvl = profile.calculate_level_info();
        assert_eq!(lvl.level, 10);
        assert_eq!(lvl.title, "Mestre Esmeralda WhatsApp");
    }

    #[test]
    fn persistence_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut a = DuckiveProfile::load_or_create(dir.path(), "Meu Pato").unwrap();
        a.record_transfer(50, 10_000_000, true, 12.0, false);
        a.equip_item("hoodie_ducker").unwrap();
        a.save(dir.path()).unwrap();

        let b = DuckiveProfile::load(dir.path())
            .unwrap()
            .expect("profile exists");
        assert_eq!(b.name, "Meu Pato");
        assert_eq!(b.files_sent_count, 50);
        assert_eq!(b.equipped.outfit.as_deref(), Some("hoodie_ducker"));
        assert!(b.unlocked_items.contains(&"hoodie_ducker".to_string()));
    }

    #[test]
    fn zero_transfer_does_not_increment_counter() {
        let mut profile = DuckiveProfile::new("ZeroCheck");
        let initial_transfers = profile.transfers_count;
        let initial_files = profile.files_sent_count;

        let summary = profile.record_transfer(0, 0, false, 0.0, false);
        assert_eq!(summary.newly_unlocked_items.len(), 0);
        assert_eq!(summary.newly_unlocked_achievements.len(), 0);
        assert_eq!(profile.transfers_count, initial_transfers);
        assert_eq!(profile.files_sent_count, initial_files);
    }

    #[test]
    fn reconcile_catalog_preserves_custom_items_and_unlocks_achievements() {
        let mut profile = DuckiveProfile::new("ReconcileTest");
        profile.files_sent_count = 120;
        profile.transfers_count = 5;
        profile
            .unlocked_items
            .push("custom_community_hat".to_string());

        profile.reconcile_catalog();

        // Verifica que itens por contagem de arquivos foram liberados
        assert!(profile
            .unlocked_items
            .contains(&"business_suit".to_string()));
        // Verifica que o item customizado prévio foi preservado
        assert!(profile
            .unlocked_items
            .contains(&"custom_community_hat".to_string()));
        // Verifica que a conquista Centurião foi retroativamente ativada
        assert!(profile
            .achievements
            .iter()
            .any(|a| a.id == "centurion" && a.unlocked));
        assert!(profile
            .achievements
            .iter()
            .any(|a| a.id == "first_flight" && a.unlocked));
    }
}
