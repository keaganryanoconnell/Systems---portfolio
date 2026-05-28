//! IPC namespace isolation — System V IPC and POSIX message queues.
//! Creation is handled via CLONE_NEWIPC. No additional setup needed
//! beyond the clone call.
