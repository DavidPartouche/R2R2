use crate::render_manager::RenderManager;
use gltf::{buffer, image, Document};
use std::cell::RefCell;
use std::ops::Index;
use std::rc::Rc;
use vulkan_ray_tracing::geometry_instance::GeometryInstanceBuilder;
use vulkan_ray_tracing::glm;

#[repr(C)]
struct Vertex {
    pub pos: glm::Vec3,
    pub norm: glm::Vec3,
    pub tex_coord: glm::Vec2,
}

#[repr(C)]
struct Material {
    pub base_color_factor: glm::Vec4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    _padding: [f32; 2],
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color_factor: glm::vec4(0.7, 0.7, 0.7, 1.0),
            metallic_factor: 0.0,
            roughness_factor: 0.0,
            _padding: [0.0, 0.0],
        }
    }
}

struct Mesh {
    indices: Vec<u32>,
    vertices: Vec<Vertex>,
    pub mat_id: u32,
}

struct Scene {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}

pub struct SceneManager {
    render_manager: Rc<RefCell<RenderManager>>,
    document: Document,
    buffers: Vec<buffer::Data>,
    images: Vec<image::Data>,
    current_scene: Option<Scene>,
}

impl SceneManager {
    pub fn new(filename: &str, render_manager: Rc<RefCell<RenderManager>>) -> Self {
        let (document, buffers, images) = gltf::import(filename).expect("GLTF file invalid");
        Self {
            render_manager,
            document,
            buffers,
            images,
            current_scene: None,
        }
    }

    pub fn load_default_scene(&mut self) {
        self.current_scene = Some(self.load_scene());
        let mesh = self.current_scene.as_ref().unwrap().meshes.index(0);
        self.load_geometry(mesh);
    }

    fn load_scene(&self) -> Scene {
        let mut meshes = Vec::with_capacity(self.document.meshes().len());
        for mesh in self.document.meshes() {
            for primitive in mesh.primitives() {
                let positions = self.get_semantic_buffer(&primitive, &gltf::Semantic::Positions, 0);

                let normals =
                    self.get_semantic_buffer(&primitive, &gltf::Semantic::Normals, positions.len());

                let tex_coord = self.get_semantic_buffer(
                    &primitive,
                    &gltf::Semantic::TexCoords(0),
                    positions.len(),
                );

                let material = primitive.material().index().unwrap_or(0);

                let mut vertices = Vec::with_capacity(positions.len() / 3);
                for i in 0..positions.len() / 3 {
                    let vertex = Vertex {
                        pos: glm::vec3(
                            positions[i * 3],
                            positions[i * 3 + 1],
                            positions[i * 3 + 2],
                        ),
                        norm: glm::vec3(normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]),
                        tex_coord: glm::vec2(tex_coord[i * 2], tex_coord[i * 2 + 1]),
                    };
                    vertices.push(vertex);
                }

                let indices = self.get_indices(&primitive, vertices.len());

                let mesh = Mesh {
                    indices,
                    vertices,
                    mat_id: material as u32,
                };
                meshes.push(mesh);
            }
        }

        let mut materials = Vec::with_capacity(self.document.materials().len());
        for material in self.document.materials() {
            let mat = Material {
                base_color_factor: glm::make_vec4(
                    &material.pbr_metallic_roughness().base_color_factor(),
                ),
                metallic_factor: material.pbr_metallic_roughness().metallic_factor(),
                roughness_factor: material.pbr_metallic_roughness().roughness_factor(),
                _padding: [0.0, 0.0],
            };
            materials.push(mat);
        }

        if self.document.materials().len() == 0 {
            materials.push(Material::default());
        }

        Scene { meshes, materials }
    }

    fn load_geometry(&self, mesh: &Mesh) {
        let size = mesh.vertices.len() * std::mem::size_of::<Vertex>();
        let vertex_buffer =
            unsafe { std::slice::from_raw_parts(mesh.vertices.as_ptr() as *const u8, size) };

        let materials = &self.current_scene.as_ref().unwrap().materials;
        let size = materials.len() * std::mem::size_of::<Material>();
        let material_buffer =
            unsafe { std::slice::from_raw_parts(materials.as_ptr() as *const u8, size) };

        // Build Geometry Instance
        let geom = GeometryInstanceBuilder::new(&self.render_manager.borrow().get_context())
            .with_vertices(vertex_buffer, mesh.vertices.len())
            .with_indices(&mesh.indices)
            .with_materials(material_buffer)
            //            .with_textures(&mut model.textures)
            .build()
            .unwrap();

        self.render_manager.borrow_mut().load_geometry(geom);
    }

    fn get_indices(&self, primitive: &gltf::Primitive, vertex_count: usize) -> Vec<u32> {
        match primitive.indices() {
            Some(accessor) => {
                let (indices_buffer, indices_count) = self.get_buffer_from_accessor(&accessor);
                unsafe {
                    std::slice::from_raw_parts(indices_buffer.as_ptr() as *const u16, indices_count)
                        .iter()
                        .map(|i| *i as u32)
                        .collect()
                }
            }
            None => (0..vertex_count).map(|i| i as u32).collect(),
        }
    }

    fn get_semantic_buffer(
        &self,
        primitive: &gltf::Primitive,
        semantic: &gltf::Semantic,
        position_count: usize,
    ) -> Vec<f32> {
        match self.find_accessor(&primitive, semantic) {
            Some(accessor) => {
                let (data, data_count) = self.get_buffer_from_accessor(&accessor);
                unsafe {
                    std::slice::from_raw_parts(data.as_ptr() as *const f32, data_count).to_vec()
                }
            }
            None => match semantic {
                gltf::Semantic::Normals => (0..position_count).map(|_| 0.0).collect(),
                gltf::Semantic::TexCoords(_) => (0..position_count * 2 / 3).map(|_| 0.0).collect(),
                _ => unreachable!(),
            },
        }
    }

    fn find_accessor<'a>(
        &self,
        primitive: &'a gltf::Primitive,
        semantic: &gltf::Semantic,
    ) -> Option<gltf::Accessor<'a>> {
        primitive.attributes().find_map(|(sem, accessor)| {
            if sem == *semantic {
                Some(accessor)
            } else {
                None
            }
        })
    }

    fn get_buffer_from_accessor(&self, accessor: &gltf::Accessor) -> (Vec<u8>, usize) {
        let buffer_view = accessor.view();
        let size = buffer_view.length();
        let offset = buffer_view.offset();
        let buffer_index = buffer_view.buffer().index();
        let buffer = &self.buffers[buffer_index];
        let positions = &buffer[offset..(offset + size)];

        let result = Vec::from(positions);

        let count = match accessor.dimensions() {
            gltf::accessor::Dimensions::Scalar => accessor.count(),
            gltf::accessor::Dimensions::Vec2 => accessor.count() * 2,
            gltf::accessor::Dimensions::Vec3 => accessor.count() * 3,
            gltf::accessor::Dimensions::Vec4 => accessor.count() * 4,
            gltf::accessor::Dimensions::Mat2 => accessor.count() * 4,
            gltf::accessor::Dimensions::Mat3 => accessor.count() * 9,
            gltf::accessor::Dimensions::Mat4 => accessor.count() * 16,
        };

        (result, count)
    }
}
