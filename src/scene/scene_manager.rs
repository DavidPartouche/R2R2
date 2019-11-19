use crate::render_manager::RenderManager;
use gltf::{buffer, image, Document};
use std::cell::RefCell;
use std::rc::Rc;
use vulkan_ray_tracing::geometry_instance::GeometryInstanceBuilder;
use vulkan_ray_tracing::glm;

#[repr(C)]
pub struct Vertex {
    pub pos: glm::Vec3,
    pub norm: glm::Vec3,
    //    pub color: glm::Vec3,
    //    pub tex_coord: glm::Vec2,
    //    pub mat_id: i32,
}

struct Mesh {
    transform: glm::Mat4,
    indices: Vec<u16>,
    vertices: Vec<Vertex>,
}

struct Scene {
    meshes: Vec<Mesh>,
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
        let default_scene = self
            .document
            .default_scene()
            .unwrap_or_else(|| self.document.scenes().nth(0).unwrap());

        self.current_scene = Some(self.load_scene(&default_scene));

        let mesh = self.current_scene.as_ref().unwrap().meshes.get(0).unwrap();

        let size = mesh.vertices.len() * std::mem::size_of::<Vertex>();
        let vertex_buffer =
            unsafe { std::slice::from_raw_parts(mesh.vertices.as_ptr() as *const u8, size) };

        // Build Geometry Instance
        let geom = GeometryInstanceBuilder::new(&self.render_manager.borrow().get_context())
            .with_vertices(vertex_buffer, mesh.vertices.len())
            .with_indices(&mesh.indices)
            //            .with_materials(&mut model.materials)
            //            .with_textures(&mut model.textures)
            .build()
            .unwrap();

        self.render_manager.borrow_mut().load_geometry(geom);
    }

    fn load_scene(&self, scene: &gltf::Scene) -> Scene {
        let mut meshes = vec![];
        for node in scene.nodes() {
            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    let indices = self.get_indices(&primitive);

                    let positions = self.get_semantic_buffer(&primitive, gltf::Semantic::Positions);
                    let normals = self.get_semantic_buffer(&primitive, gltf::Semantic::Normals);
                    let mut vertices = vec![];
                    for i in 0..positions.len() / 3 {
                        let vertex = Vertex {
                            pos: glm::vec3(
                                positions[i * 3],
                                positions[i * 3 + 1],
                                positions[i * 3 + 2],
                            ),
                            norm: glm::vec3(normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]),
                        };
                        vertices.push(vertex);
                    }

                    let mesh = Mesh {
                        transform: glm::identity(),
                        indices,
                        vertices,
                    };
                    meshes.push(mesh);
                }
            }
        }

        Scene { meshes }
    }

    fn get_indices(&self, primitive: &gltf::Primitive) -> Vec<u16> {
        let (indices_buffer, indices_count) =
            self.get_buffer_from_accessor(&primitive.indices().unwrap());
        unsafe {
            std::slice::from_raw_parts(indices_buffer.as_ptr() as *const u16, indices_count)
                .to_vec()
        }
    }

    fn get_semantic_buffer(
        &self,
        primitive: &gltf::Primitive,
        semantic: gltf::Semantic,
    ) -> Vec<f32> {
        let accessor = self.find_accessor(&primitive, semantic).unwrap();
        let (data, data_count) = self.get_buffer_from_accessor(&accessor);
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const f32, data_count).to_vec() }
    }

    fn find_accessor<'a>(
        &self,
        primitive: &'a gltf::Primitive,
        semantic: gltf::Semantic,
    ) -> Option<gltf::Accessor<'a>> {
        primitive.attributes().find_map(|(sem, accessor)| {
            if sem == semantic {
                Some(accessor)
            } else {
                None
            }
        })
    }

    fn get_buffer_from_accessor(&self, accessor: &gltf::Accessor) -> (Vec<u8>, usize) {
        let mut result = vec![];

        let buffer_view = accessor.view();
        let size = buffer_view.length();
        let offset = buffer_view.offset();
        let buffer_index = buffer_view.buffer().index();
        let buffer = &self.buffers[buffer_index];
        let positions = &buffer[offset..(offset + size)];

        result.extend_from_slice(positions);
        (result, accessor.count())
    }
}
