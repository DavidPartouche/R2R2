use std::mem;
use std::rc::Rc;

use crate::acceleration_structure::{AccelerationStructureBuilder, Instance};
use crate::bottom_level_acceleration_structure::{
    BottomLevelAccelerationStructure, BottomLevelAccelerationStructureBuilder,
};
use crate::errors::VulkanError;
use crate::geometry_instance::{GeometryInstance, GeometryInstanceBuilder};
use crate::ray_tracing::{RayTracing, RayTracingBuilder};
use crate::vertex::Vertex;
use crate::vulkan_context::VulkanContext;

pub struct RayTracingPipeline {
    ray_tracing: Rc<RayTracing>,
}

pub struct RayTracingPipelineBuilder<'a> {
    context: &'a VulkanContext,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl<'a> RayTracingPipelineBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        RayTracingPipelineBuilder {
            context,
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn with_vertices(mut self, vertices: &mut Vec<Vertex>) -> Self {
        self.vertices.append(vertices);
        self
    }

    pub fn with_indices(mut self, indices: &mut Vec<u32>) -> Self {
        self.indices.append(indices);
        self
    }

    pub fn build(mut self) -> Result<RayTracingPipeline, VulkanError> {
        let ray_tracing = Rc::new(RayTracingBuilder::new(&self.context).build()?);

        let geom = GeometryInstanceBuilder::new(&self.context)
            .with_vertices(&mut self.vertices)
            .with_indices(&mut self.indices)
            .build();

        self.create_acceleration_structures(Rc::clone(&ray_tracing), &geom);

        Ok(RayTracingPipeline { ray_tracing })
    }

    fn create_acceleration_structures(&self, ray_tracing: Rc<RayTracing>, geom: &GeometryInstance) {
        let command_buffer = self.context.begin_single_time_commands().unwrap();

        let bottom_level_as = vec![self.create_bottom_level_as(geom)];
        let structure = AccelerationStructureBuilder::new(&self.context, Rc::clone(&ray_tracing))
            .with_bottom_level_as(&bottom_level_as)
            .with_command_buffer(command_buffer)
            .build()
            .unwrap();

        let instances: Vec<Instance> = vec![Instance {
            bottom_level_as: structure.get(),
            transform: geom.transform,
            instance_id: 0,
            hit_group_index: 0,
        }];

        let top_level_as =
            AccelerationStructureBuilder::new(&self.context, Rc::clone(&ray_tracing))
                .with_top_level_as(&instances)
                .with_command_buffer(command_buffer)
                .build()
                .unwrap();

        self.context
            .end_single_time_commands(command_buffer)
            .unwrap();
    }

    fn create_bottom_level_as(&self, geom: &GeometryInstance) -> BottomLevelAccelerationStructure {
        BottomLevelAccelerationStructureBuilder::new()
            .with_vertex_buffer(geom.vertex_buffer.get())
            .with_vertex_offset(geom.vertex_offset)
            .with_vertex_count(geom.vertex_count as u32)
            .with_vertex_size(mem::size_of::<Vertex>() as u32)
            .with_index_buffer(geom.index_buffer.get())
            .with_index_offset(geom.index_offset)
            .with_index_count(geom.index_count as u32)
            .build()
    }
}
