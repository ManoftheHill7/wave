use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/generated_items.rs"));

#[derive(Debug, Clone, Copy)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub count: u32,
}

impl ItemStack {
    pub fn new(item_type: ItemType, count: u32) -> Self {
        ItemStack { item_type, count }
    }

    pub fn total_weight(&self) -> f32 {
        self.item_type.weight() * self.count as f32
    }
}

pub struct Inventory {
    items: HashMap<ItemType, u32>,
    max_weight: f32,
}

impl Inventory {
    pub fn new(max_weight: f32) -> Self {
        Inventory {
            items: HashMap::new(),
            max_weight,
        }
    }

    pub fn has(&self, item: ItemType, count: u32) -> bool {
        self.items
            .get(&item)
            .map_or(false, |&amount| amount >= count)
    }

    pub fn add(&mut self, item: ItemType, count: u32) -> u32 {
        if count == 0 {
            return 0;
        }

        let item_weight = item.weight();
        let current_weight = self.current_weight();
        let available_weight = self.max_weight - current_weight;

        if available_weight <= 0.0 {
            return 0;
        }

        let max_addable = (available_weight / item_weight).floor() as u32;
        let actual_add = count.min(max_addable);

        if actual_add > 0 {
            *self.items.entry(item).or_insert(0) += actual_add;
        }

        actual_add
    }

    pub fn take(&mut self, item: ItemType, count: u32) -> u32 {
        if count == 0 {
            return 0;
        }

        let available = self.items.get(&item).copied().unwrap_or(0);
        let taken = count.min(available);

        if taken > 0 {
            let remaining = available - taken;
            if remaining == 0 {
                self.items.remove(&item);
            } else {
                self.items.insert(item, remaining);
            }
        }

        taken
    }

    pub fn count(&self, item: ItemType) -> u32 {
        self.items.get(&item).copied().unwrap_or(0)
    }

    pub fn current_weight(&self) -> f32 {
        self.items
            .iter()
            .map(|(item_type, count)| item_type.weight() * (*count as f32))
            .sum()
    }

    pub fn available_weight(&self) -> f32 {
        self.max_weight - self.current_weight()
    }

    pub fn max_weight(&self) -> f32 {
        self.max_weight
    }

    pub fn unique_items(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear()
    }

    pub fn iter(&self) -> impl Iterator<Item = ItemStack> + '_ {
        self.items.iter().map(|(item_type, count)| ItemStack {
            item_type: *item_type,
            count: *count,
        })
    }
}
