use crate::winit::event::Event;
use hex::{
    assets::{Mesh, Texture, mesh::Vertex3},
    components::{Camera3, Light3, Model, Trans3},
    context::Context3,
    nalgebra::*,
    renderers::ModelRenderer,
    vulkano::{image::sampler::Sampler, swapchain::PresentMode},
    winit::{event::WindowEvent, event_loop::EventLoop, window::WindowBuilder},
    world::entity_manager::*,
    *,
};
use image::{ImageFormat, ImageReader};
use std::sync::{Arc, RwLock};
use std::fs::File;
use std::{io::BufReader, sync::LazyLock};

struct Sys {
    pub last_frame: std::time::Instant,
}

fn update(ctrl: Arc<RwLock<Control>>, world: Arc<RwLock<World>>) -> anyhow::Result<()> {
    static LAST_FRAME: LazyLock<RwLock<std::time::Instant>> =
        LazyLock::new(|| RwLock::new(std::time::Instant::now()));

    if matches!(
        ctrl.read().unwrap().event,
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        }
    ) {
        let mut last_frame = LAST_FRAME.write().unwrap();
        let frame = std::time::Instant::now();
        let delta = frame.duration_since(*last_frame);

        *last_frame = frame;

        let em = world.read().unwrap().em.clone();

        em.read()
            .unwrap()
            .entities()
            .filter(|e| em.read().unwrap().get_component::<Model>(*e).is_some())
            .for_each(|e| {
                if let Some(transform) = em.read().unwrap().get_component::<Trans3>(e) {
                    let transform = &mut *transform.write().unwrap();

                    transform.set_rotation(
                        transform.rotation() + Vector3::new(1.0, 1.0, 1.0) * delta.as_secs_f32(),
                    );
                }
            });
    }

    Ok(())
}

fn main() {
    let ev = EventLoop::new().unwrap();
    let wb = Arc::new(
        WindowBuilder::new()
            .with_title("Paraselene Reimagined")
            .build(&ev)
            .unwrap(),
    );
    let context =
        Context3::new(&ev, wb, PresentMode::Fifo, Vector4::new(0.0, 0.0, 0.0, 1.0)).unwrap();
    let em: Arc<RwLock<EntityManager>> = EntityManager::new();

    {
        let mut em = em.write().unwrap();
        let camera = em.add(true);

        em.add_component(camera, Camera3::new(16.0 / 9.0, 90.0, 0.1, 1000.0));
        em.add_component(
            camera,
            Trans3::new(
                Vector3::new(0.0, 0.0, -10.0),
                Vector3::zeros(),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );

        let light = em.add(true);

        em.add_component(
            light,
            Light3::new(
                Vector3::from([10.0; 3]),
                Vector3::new(1.0, 1.0, 1.0),
                1.0,
                32.0,
                &context.read().unwrap(),
            )
            .unwrap(),
        );
        em.add_component(
            light,
            Trans3::new(
                Vector3::new(-100.0, 0.0, -50.0),
                Vector3::zeros(),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );

        let light2 = em.add(true);

        em.add_component(
            light2,
            Light3::new(
                Vector3::from([10.0; 3]),
                Vector3::new(1.0, 1.0, 1.0),
                1.0,
                32.0,
                &context.read().unwrap(),
            )
            .unwrap(),
        );
        em.add_component(
            light2,
            Trans3::new(
                Vector3::new(-100.0, 100.0, -50.0),
                Vector3::zeros(),
                Vector3::new(1.0, 1.0, 1.0),
            ),
        );
    }

    let (vertices, indices) = {
        let input = BufReader::new(File::open("teapot.obj").unwrap());
        let obj: obj::Obj<obj::TexturedVertex> = obj::load_obj(input).unwrap();

        let verts = obj
            .vertices
            .into_iter()
            .map(|v| Vertex3 {
                position: v.position,
                normal: {
                    println!("{:?}", v.normal);

                    v.normal
                },
                color: <[f32; 4]>::from(Vector4::new(1.0, 1.0, 1.0, 1.0)),
                uv: Vector2::new(v.texture[0], v.texture[1]).into(),
            })
            .collect::<Vec<_>>();

        let inds = obj
            .indices
            .into_iter()
            .map(|i| i as u32)
            .collect::<Vec<_>>();

        (verts, inds)
    };

    let model = Model::new(
        Mesh::new(&vertices, &indices, &context.read().unwrap()).unwrap(),
        load_texture("texture.png", &context.read().unwrap()).unwrap(),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        &context.read().unwrap(),
    )
    .unwrap();

    for _i in 0..50_000 {
        let mut em = em.write().unwrap();
        let e = em.add(true);

        em.add_component(
            e,
            Trans3::new(
                Vector3::new(
                    (5 * rand::random_range(-100..100)) as f32,
                    (5 * rand::random_range(-100..100)) as f32,
                    (5 * rand::random_range(-100..100)) as f32 - 100.0,
                ),
                Vector3::zeros(),
                Vector3::new(5.0, 5.0, 5.0),
            ),
        );

        em.add_component(e, model.clone());
    }

    let world = world::World::new(
        world::entity_manager::EntityManager::new(),
        Vector3::new(0.1, 0.1, 0.1),
        32.0,
    );
    let event_loop = EventLoop::new().unwrap();
    let mut model_renderer =
        ModelRenderer::new(Vector4::new(0.0, 0.0, 0.0, 1.0), &context.read().unwrap()).unwrap();

    Context3::init(
        context,
        world,
        event_loop,
        move |context: Arc<RwLock<Context3>>,
              world: Arc<RwLock<World>>,
              ctrl: Arc<RwLock<Control>>,
              recreate_swapchain: &mut bool| {
            update(ctrl.clone(), world.clone())?;

            model_renderer.draw(
                context.clone(),
                world.clone(),
                ctrl.clone(),
                recreate_swapchain,
            )?;

            Ok(())
        },
    )
    .unwrap();
}

pub fn load_texture(path: &str, context: &Context3) -> anyhow::Result<Texture> {
    let mut img = ImageReader::open(path)?;

    img.set_format(ImageFormat::Png);

    let img = img.decode().unwrap().to_rgba8();
    let dims = img.dimensions();
    let img = img.into_raw();
    let sampler = Sampler::new(context.device.clone(), Default::default()).unwrap();

    Ok(Texture::new(sampler, &img, dims.0, dims.1, context).unwrap())
}
