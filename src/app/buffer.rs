use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device, ShaderStages};

pub struct BindGroupBuilder<'a> {
    name: String,
    layout_entries: Vec<BindGroupLayoutEntry> = vec![],
    entries: Vec<BindGroupEntry<'a>> = vec![],
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new(name: String) -> Self {
        Self { name, .. }
    }

    pub fn add_layout_entry(&mut self, visibility: ShaderStages, ty: BindingType) -> &mut Self {
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.layout_entries.len() as u32,
            visibility, ty,
            count: None
        });
        self
    }

    pub fn add_entry(&mut self, resource: BindingResource<'a>) -> &mut Self {
        self.entries.push(BindGroupEntry {
            binding: self.entries.len() as u32,
            resource
        });
        self
    }

    pub fn build(&mut self, device: &Device) -> (BindGroupLayout, BindGroup) {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some((self.name.clone() + "_bind_group_layout").as_str()),
            entries: self.layout_entries.as_slice()
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some((self.name.clone() + "_bind_group").as_str()),
            entries: self.entries.as_slice(),
            layout: &layout
        });
        (layout, bind_group)
    }

    pub fn build_layout(&mut self, device: &Device) -> BindGroupLayout {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some((self.name.clone() + "_bind_group_layout").as_str()),
            entries: self.layout_entries.as_slice()
        });
        layout
    }

    pub fn build_bind_group(&mut self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some((self.name.clone() + "_bind_group").as_str()),
            entries: self.entries.as_slice(),
            layout: &layout
        });
        bind_group
    }
}