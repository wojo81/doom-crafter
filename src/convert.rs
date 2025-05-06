use three_d::*;

pub fn convert_all(paths_and_names: impl Iterator<Item = (String, String)>) -> anyhow::Result<()> {
    let viewport = Viewport::new_at_origo(64, 64);
    let context = HeadlessContext::new()?;
    let camera = Camera::new_perspective(
        viewport,
        vec3(0.0, 0.0, 2.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(60.0),
        0.1,
        100.0,
    );

    for (path, name) in paths_and_names {
        convert(path, name, &viewport, &context, &camera)?;
    }

    Ok(())
}

fn convert(path: String, name: String, viewport: &Viewport, context: &Context, camera: &Camera) -> anyhow::Result<()> {
    let mut prism = Prism::load(path, name.clone(), Patch::BODY, Vec3::zero(), &context)?;

    let mut target_texture = Texture2D::new_empty::<[u8; 4]>(
        &context,
        viewport.width,
        viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );

    let mut depth_texture = DepthTexture2D::new::<f32>(
        &context,
        viewport.width,
        viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
    );
    for frame_index in 0..8 {
        for face in prism.faces.iter_mut() {
            face.model.set_transformation(Mat4::from_translation(vec3(0.0, 0.0, -20.0)) * Mat4::from_angle_x(degrees(frame_index as f32 * 45.0)));
        }

        let pixels = RenderTarget::new(target_texture.as_color_target(None), depth_texture.as_depth_target())
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 0.0, 1.0))
            .render(
                &camera,
                prism.faces[0].model
                    .into_iter()
                    .chain(&prism.faces[1].model)
                    .chain(&prism.faces[2].model)
                    .chain(&prism.faces[3].model)
                    .chain(&prism.faces[4].model)
                    .chain(&prism.faces[5].model),
                &[]
            )
            .read_color();

        use three_d_asset::io::Serialize;

        three_d_asset::io::save(
            &CpuTexture {
                data: TextureData::RgbaU8(pixels),
                width: viewport.width,
                height: viewport.height,
                ..Default::default()
            }
            .serialize(format!("out/{name}-{frame_index}.png"))
            .unwrap(),
        )
        .unwrap();
    }

    Ok(())
}

struct Prism {
    faces: [Face; 6],
    width: u32,
    height: u32,
    depth: u32,
}

impl Prism {
    fn load(path: String, name: String, patch: Patch, center: Vec3, context: &Context) -> anyhow::Result<Self> {
        let atlas = image::open(&path)?;
        Ok(Self {
            faces: [
                Face::new_front(&name, &atlas, patch, center,  context),
                Face::new_right(&name, &atlas, patch, center, context),
                Face::new_back(&name, &atlas, patch, center, context),
                Face::new_left(&name, &atlas, patch, center, context),
                Face::new_top(&name, &atlas, patch, center, context),
                Face::new_bottom(&name, &atlas, patch, center, context)
            ], width: patch.width, height: patch.height, depth: patch.depth
        })
    }
}

struct Face {
    model: Gm<Mesh, ColorMaterial>,

    width: u32,
    height: u32,
    depth: u32,
}

impl Face {
    fn new_front(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, z) + center,
            vec3(x, -y, z) + center,
            vec3(x, y, z) + center,
            vec3(-x, y, z) + center
        ];
        Self::new(name, atlas, patch, positions, 0, context)
    }

    fn new_left(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(x, -y, z) + center,
            vec3(x, -y, -z) + center,
            vec3(x, y, -z) + center,
            vec3(x, y, z) + center
        ];
        Self::new(name, atlas, patch.as_right(), positions, 1, context)
    }

    fn new_back(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(x, -y, -z) + center,
            vec3(-x, -y, -z) + center,
            vec3(-x, y, -z) + center,
            vec3(x, y, -z) + center
        ];
        Self::new(name, atlas, patch.as_back(), positions, 2, context)
    }

    fn new_right(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z) + center,
            vec3(-x, -y, z) + center,
            vec3(-x, y, z) + center,
            vec3(-x, y, -z) + center
        ];
        Self::new(name, atlas, patch.as_left(), positions, 2, context)
    }

    fn new_top(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, y, z) + center,
            vec3(x, y, z) + center,
            vec3(x, y, -z) + center,
            vec3(-x, y, -z) + center
        ];
        Self::new(name, atlas, patch.as_top(), positions, 2, context)
    }

    fn new_bottom(name: &str, atlas: &image::DynamicImage, patch: Patch, center: Vec3, context: &Context) -> Self {
        let Vector3 { x, y, z } = patch.half_size();
        let positions = vec![
            vec3(-x, -y, -z) + center,
            vec3(x, -y, -z) + center,
            vec3(x, -y, z) + center,
            vec3(-x, -y, z) + center
        ];
        Self::new(name, atlas, patch.as_bottom(), positions, 2, context)
    }

    fn new(name: &str, atlas: &image::DynamicImage, patch: Patch, positions: Vec<Vec3>, index: u8, context: &Context) -> Self {
        let indices = vec![0, 1, 2, 2, 3, 0];

        let uvs = vec![
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.0),
        ];

        let mesh = CpuMesh {
            positions: Positions::F32(positions),
            indices: Indices::U8(indices),
            uvs: Some(uvs),
            ..Default::default()
        };

        let sub_image = image::imageops::crop_imm(atlas, patch.x, patch.y, patch.width, patch.height).to_image();
        let data = TextureData::RgbaU8(
            sub_image.into_raw()
                .chunks(4)
                .map(|p| [p[0], p[1], p[2], p[3]])
                .collect()
        );
        let texture = CpuTexture {
            name: format!("{name}_{index}"),
            data,
            width: patch.width,
            height: patch.height,
            min_filter: Interpolation::Nearest,
            mag_filter: Interpolation::Nearest,
            ..Default::default()
        };

        let model = Gm::new(
            Mesh::new(context, &mesh),
            ColorMaterial {
                texture: Some(Texture2DRef::from_cpu_texture(&context, &texture)),
                render_states: RenderStates { cull: Cull::Back, ..Default::default() },
                ..Default::default()
            },
        );

        Self { model, width: patch.width, height: patch.height, depth: patch.depth }
    }
}

#[derive(Clone, Copy)]
struct Patch {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    depth: u32,
}

impl Patch {
    const HEAD: Patch = Patch::new(8, 8, 8, 8, 8);
    const BODY: Patch = Patch::new(20, 20, 8, 12, 4);

    const fn new(x: u32, y: u32, width: u32, height: u32, depth: u32) -> Self {
        Self { x, y, width, height, depth }
    }

    fn half_size(&self) -> Vec3 {
        vec3(self.width as f32 / 2.0, self.height as f32 / 2.0, self.depth as f32 / 2.0)
    }

    fn as_right(&self) -> Self {
        Self::new(self.x + self.width, self.y, self.depth, self.height, self.width)
    }

    fn as_back(&self) -> Self {
        Self::new(self.x + self.width + self.depth, self.y, self.width, self.height, self.depth)
    }

    fn as_left(&self) -> Self {
        Self::new(self.x - self.depth, self.y, self.depth, self.height, self.width)
    }

    fn as_top(&self) -> Self {
        Self::new(self.x, self.y - self.depth, self.width, self.depth, self.height)
    }

    fn as_bottom(&self) -> Self {
        Self::new(self.x + self.width, self.y - self.depth, self.width, self.depth, self.height)
    }
}