fn spawn(mut commands: Commands) {
    let mass = Mass::kilograms(1.49828e+10);
    let positions = vec![
        VecLength::meters(0.974545, -0.238734),
        VecLength::meters(-0.965221, 0.247381),
        VecLength::meters(-0.00932407, -0.00864713),
    ];
    let velocities = vec![
        VecVelocity::meters_per_second(0.454079, 0.435395),
        VecVelocity::meters_per_second(0.478329, 0.429318),
        VecVelocity::meters_per_second(-0.932407, -0.864713),
    ];
    for (pos, vel) in positions.into_iter().zip(velocities) {
        spawn_particle(&mut commands, pos, vel, mass)
    }
}


fn spawn_particle(commands: &mut Commands, pos: VecLength, vel: VecVelocity, mass: Mass) {
    commands.spawn_bundle((
        LocalParticle,
        components::Position(pos),
        components::Velocity(vel),
        components::Mass(mass),
    ));
}
