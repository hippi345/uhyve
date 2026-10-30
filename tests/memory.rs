mod common;

use std::thread;

use common::{
	BuildMode, build_hermit_bin, check_result_and_print_output, env_logger_build,
	run_vm_with_custom_memory, run_vm_with_custom_memory_params,
};
use uhyvelib::VmResult;

const SERIAL_KERNEL_OUTPUT: &str = "Hello from serial!\nABCD\n1234ASDF!@#$\n";

/// Runs the `hello_world` kernel with the given memory size.
#[inline]
fn run_x_mib(memory_size: u64) -> VmResult {
	env_logger_build();
	let bin_path = build_hermit_bin("hello_world", BuildMode::Debug);
	run_vm_with_custom_memory(bin_path, memory_size)
}

/// Tries to start Uhyve VMs with different memory sizes to evaluate if the
/// run will succeed.
#[test]
fn memory_test() {
	// u64: memory (MiB), bool: success expected
	let params = vec![
		(128, true),
		(256, true),
		(512, true),
		(1024, true),
		(2048, true),
		#[cfg(target_os = "linux")]
		(3072, false), // v1 (KVM_32BIT_GAP_START)
		(4096, false), // v1
	];
	for (mem_size, success_expected) in params {
		let vm = thread::spawn(move || run_x_mib(mem_size));
		println!("MiB: {mem_size}, success: {success_expected}");
		if success_expected {
			let res = vm.join().expect("VM panicked for {mem_size}MiB");
			check_result_and_print_output(&res, 0);
		} else {
			vm.join().expect_err("VM did not panic for {mem_size}MiB.");
		}
	}
}

/// Regression coverage for large guest RAM with an SMP kernel (see hermit-os/uhyve#1081).
#[test]
fn memory_serial_large_guest_test() {
	env_logger_build();
	let bin_path = build_hermit_bin("serial", BuildMode::Debug);

	for mem_size in [2816_u64, 3000, 3072] {
		let bin_path = bin_path.clone();
		let vm = thread::spawn(move || {
			run_vm_with_custom_memory_params(bin_path, mem_size, |params| {
				params.cpu_count = 2.try_into().unwrap();
			})
		});
		println!("serial kernel, MiB: {mem_size}");
		let res = vm
			.join()
			.expect("VM thread panicked for {mem_size} MiB serial test");
		check_result_and_print_output(&res, 0);
		assert!(
			res.output
				.as_ref()
				.is_some_and(|output| output.contains(SERIAL_KERNEL_OUTPUT)),
			"serial kernel output missing at {mem_size} MiB"
		);
	}
}
