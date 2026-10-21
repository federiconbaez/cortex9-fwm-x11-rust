//! fwm: gestor de ventanas X11 para orquestación cognitiva.
//! Las ventanas se agrupan por afinidad semántica y se exponen a los agentes por un bus IPC.
pub mod affinity;
pub mod ipc;
pub mod layout;
pub mod vision;
