use std::marker::PhantomData;
use std::sync::mpsc::channel;

use bevy::prelude::Plugin;

struct CommunicationPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T: Sync + Send + 'static> Plugin for CommunicationPlugin<T> {
    fn build(&self, app: &mut bevy::prelude::App) {
        add_communicator_to_app::<T>(app);
    }
}

#[cfg(feature = "local")]
fn add_communicator_to_app<T>(app: &mut bevy::prelude::App) {
    use super::ExchangeCommunicator;
    let (sender, receiver) = channel();
    app.insert_resource(ExchangeCommunicator::<T>::new());
}

#[cfg(not(feature = "local"))]
fn add_communicator_to_app<T>(app: &mut bevy::prelude::App) {
    todo!()
}
