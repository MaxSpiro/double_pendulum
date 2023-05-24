use bevy::{ prelude::*, sprite::MaterialMesh2dBundle };
use std::f64::consts::PI;

static G: f64 = 10f64;

// Degrees
static INITIAL_THETA_1: f64 = 15.0;
static INITIAL_THETA_2: f64 = 45.0;

static LENGTH_1: f64 = 100.0;
static LENGTH_2: f64 = 80.0;

static MASS_1: f64 = 5.0;
static MASS_2: f64 = 5.0;

fn main() {
    App::new().add_plugins(DefaultPlugins).add_startup_system(setup).add_system(system).run();
}

#[derive(Component)]
struct Ball {
    x: f32,
    y: f32,
    is_end: bool,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let pendulum = DoublePendulum::new(
        (INITIAL_THETA_1 * PI) / 180f64,
        (INITIAL_THETA_2 * PI) / 180f64,
        LENGTH_1,
        LENGTH_2,
        MASS_1,
        MASS_2
    );
    let system_height = (pendulum.length1 + pendulum.length2) as f32;
    let x1 = pendulum.x1 as f32;
    let x2 = pendulum.x2 as f32;
    let y1 = pendulum.y1 as f32;
    let y2 = pendulum.y2 as f32;
    let camera = Camera2dBundle::default();
    commands.spawn(camera);
    commands.spawn(pendulum);
    commands.spawn((
        Ball { x: x1, y: y1, is_end: false },
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(15.0).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_xyz(x1, y1, 0.0),
            ..default()
        },
    ));
    commands.spawn((
        Ball { x: x2, y: y2, is_end: true },
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(15.0).into()).into(),
            material: materials.add(ColorMaterial::from(Color::BLUE)),
            transform: Transform::from_xyz(x2, y2, 0.0),
            ..default()
        },
    ));

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(10.0).into()).into(),
        material: materials.add(ColorMaterial::from(Color::GREEN)),
        transform: Transform::from_xyz(0.0, system_height, 0.0),
        ..default()
    });
}

fn system(
    time: Res<Time>,
    mut q: Query<&mut DoublePendulum>,
    mut balls: Query<(&mut Ball, &mut Transform)>
) {
    let mut pendulum = q.single_mut();
    pendulum.next(time.delta_seconds_f64() * 8.0);
    for (mut ball, mut transform) in &mut balls {
        if ball.is_end {
            ball.x = pendulum.x2 as f32;
            ball.y = pendulum.y2 as f32;
        } else {
            ball.x = pendulum.x1 as f32;
            ball.y = pendulum.y1 as f32;
        }
        transform.translation.x = ball.x;
        transform.translation.y = ball.y;
    }
}

#[derive(Debug, Component)]
struct DoublePendulum {
    length1: f64,
    length2: f64,
    mass1: f64,
    mass2: f64,

    theta1: f64,
    omega1: f64,
    alpha1: f64,
    x1: f64,
    y1: f64,
    theta2: f64,
    omega2: f64,
    alpha2: f64,
    x2: f64,
    y2: f64,
}

impl DoublePendulum {
    fn new(
        initial_theta1: f64,
        initial_theta2: f64,
        length1: f64,
        length2: f64,
        mass1: f64,
        mass2: f64
    ) -> DoublePendulum {
        assert!(mass1 > 0f64);
        assert!(mass2 > 0f64);
        assert!(length1 > 0f64);
        assert!(length2 > 0f64);
        let system_height = length1 + length2;
        let x1 = length1 * initial_theta1.sin();
        DoublePendulum {
            length1,
            length2,
            mass1,
            mass2,
            theta1: initial_theta1,
            omega1: 0f64,
            alpha1: alpha1(
                mass1,
                mass2,
                initial_theta1,
                initial_theta2,
                0f64,
                0f64,
                length1,
                length2
            ),
            x1,
            y1: system_height - length1 * initial_theta1.cos(),
            theta2: initial_theta2,
            omega2: 0f64,
            alpha2: alpha2(
                mass1,
                mass2,
                initial_theta1,
                initial_theta2,
                0f64,
                0f64,
                length1,
                length2
            ),
            x2: x1 + length2 * initial_theta2.sin(),
            y2: system_height - (length1 * initial_theta1.cos() + length2 * initial_theta2.cos()),
        }
    }

    fn next(&mut self, dt: f64) {
        self.omega1 = self.omega1 + self.alpha1 * dt;
        self.theta1 = self.theta1 + self.omega1 * dt + 0.5 * self.alpha1 * dt.powi(2);
        self.omega2 = self.omega2 + self.alpha2 * dt;
        self.theta2 = self.theta2 + self.omega2 * dt + 0.5 * self.alpha2 * dt.powi(2);
        self.alpha1 = alpha1(
            self.mass1,
            self.mass2,
            self.theta1,
            self.theta2,
            self.omega1,
            self.omega2,
            self.length1,
            self.length2
        );
        self.alpha2 = alpha2(
            self.mass1,
            self.mass2,
            self.theta1,
            self.theta2,
            self.omega1,
            self.omega2,
            self.length1,
            self.length2
        );
        let system_height = self.length2 + self.length1;
        self.x1 = self.length1 * self.theta1.sin();
        self.y1 = system_height - self.length1 * self.theta1.cos();
        self.x2 = self.x1 + self.length2 * self.theta2.sin();
        self.y2 =
            system_height - (self.length1 * self.theta1.cos() + self.length2 * self.theta2.cos());

        self.theta1 = self.theta1 % (2.0 * PI);
        self.theta2 = self.theta2 % (2.0 * PI);
    }
}

fn alpha1(
    mass1: f64,
    mass2: f64,
    theta1: f64,
    theta2: f64,
    omega1: f64,
    omega2: f64,
    length1: f64,
    length2: f64
) -> f64 {
    let numerator =
        -1.0 * G * (2.0 * mass1 + mass2) * theta1.sin() -
        mass2 * G * (theta1 - 2.0 * theta2).sin() -
        2.0 *
            (theta1 - theta2).sin() *
            mass2 *
            (omega2.powi(2) * length2 + omega1.powi(2) * length1 * (theta1 - theta2).cos());

    let denomenator = length1 * (2.0 * mass1 + mass2 - mass2 * (2.0 * theta1 - 2.0 * theta2).cos());

    numerator / denomenator
}

fn alpha2(
    mass1: f64,
    mass2: f64,
    theta1: f64,
    theta2: f64,
    omega1: f64,
    omega2: f64,
    length1: f64,
    length2: f64
) -> f64 {
    let numerator =
        2.0 *
        (theta1 - theta2).sin() *
        (omega1.powi(2) * length1 * (mass1 + mass2) +
            G * (mass1 + mass2) * theta1.cos() +
            omega2.powi(2) * length2 * mass2 * (theta1 - theta2).cos());

    let denomenator = length2 * (2.0 * mass1 + mass2 - mass2 * (2.0 * theta1 - 2.0 * theta2).cos());

    numerator / denomenator
}