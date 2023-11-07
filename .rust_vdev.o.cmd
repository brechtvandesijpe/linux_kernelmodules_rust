cmd_samples/rust/rust_vdev.o := RUST_MODFILE=samples/rust/rust_vdev rustc --edition=2021 -Zbinary_dep_depinfo=y -Dunsafe_op_in_unsafe_fn -Drust_2018_idioms -Dunreachable_pub -Dnon_ascii_idents -Wmissing_docs -Drustdoc::missing_crate_level_docs -Dclippy::correctness -Dclippy::style -Dclippy::suspicious -Dclippy::complexity -Dclippy::perf -Dclippy::let_unit_value -Dclippy::mut_mut -Dclippy::needless_bitwise_bool -Dclippy::needless_continue -Wclippy::dbg_macro --target=./rust/target.json -Cpanic=abort -Cembed-bitcode=n -Clto=n -Cforce-unwind-tables=n -Ccodegen-units=1 -Csymbol-mangling-version=v0 -Crelocation-model=static -Zfunction-sections=n -Dclippy::float_arithmetic -Ctarget-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2 -Ztune-cpu=generic -Cno-redzone=y -Ccode-model=kernel -Cdebug-assertions=n -Coverflow-checks=y -Copt-level=2 -Cforce-frame-pointers=y -Cdebuginfo=2    @./include/generated/rustc_cfg -Zallow-features=allocator_api,bench_black_box,core_ffi_c,generic_associated_types,const_ptr_offset_from,const_refs_to_cell -Zcrate-attr=no_std -Zcrate-attr='feature(allocator_api,bench_black_box,core_ffi_c,generic_associated_types,const_ptr_offset_from,const_refs_to_cell)' --extern alloc --extern kernel --crate-type rlib --out-dir samples/rust -L ./rust/ --crate-name rust_vdev --emit=dep-info,obj samples/rust/rust_vdev.rs; mv samples/rust/rust_vdev.d samples/rust/.rust_vdev.o.d; sed -i '/^$(pound)/d' samples/rust/.rust_vdev.o.d

source_samples/rust/rust_vdev.o := samples/rust/rust_vdev.rs

deps_samples/rust/rust_vdev.o := \
  /home/user/linux/rust/libcore.rmeta \
  /home/user/linux/rust/libcompiler_builtins.rmeta \
  /home/user/linux/rust/libkernel.rmeta \
  /home/user/linux/rust/libbindings.rmeta \
  /home/user/linux/rust/libmacros.so \
  /home/user/linux/rust/liballoc.rmeta \
  /home/user/linux/rust/libbuild_error.rmeta \
  /home/user/linux/rust/libcore.rmeta \
  /home/user/linux/rust/libcompiler_builtins.rmeta \
  /home/user/linux/rust/libkernel.rmeta \
  /home/user/linux/rust/libbindings.rmeta \
  /home/user/linux/rust/libmacros.so \
  /home/user/linux/rust/liballoc.rmeta \
  /home/user/linux/rust/libbuild_error.rmeta \

samples/rust/rust_vdev.o: $(deps_samples/rust/rust_vdev.o)

$(deps_samples/rust/rust_vdev.o):
