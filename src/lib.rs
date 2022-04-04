use std::ffi::CStr;
use std::ops::DerefMut;
use interoptopus::{ffi_function, ffi_type, Inventory, InventoryBuilder, function, ffi_constant, ffi_service, ffi_service_ctor,ffi_service_method,ffi_surrogates, constant, pattern, callback};
use interoptopus::lang::c::{CType, Field, PrimitiveType, CompositeType};
use interoptopus::patterns::result::FFIError;
use interoptopus::patterns::slice::{FFISlice, FFISliceMut};
use interoptopus::patterns::string::AsciiPointer;
use interoptopus::patterns::api_guard::APIVersion;
use interoptopus::patterns::option::FFIOption;



// Guard function used by backends.
#[ffi_function]
#[no_mangle]
pub extern "C" fn my_api_guard() -> APIVersion {
    my_inventory().into()
}

#[ffi_function]
#[no_mangle]
pub extern "C" fn get_value()->FFIOption<u8> {
    FFIOption::none()
}

#[ffi_constant]
const SOME_CONST: u32 = 314;



// Some Error used in your application.
#[derive(Debug)]
pub enum MyError {
    Bad,
}

// The error FFI users should see
#[ffi_type(patterns(ffi_error))]
#[repr(C)]
pub enum MyFFIError {
    Ok = 0,
    NullPassed = 1,
    Panic = 2,
    OtherError = 3,
}

// Gives special meaning to some of your error variants.
impl FFIError for MyFFIError {
    const SUCCESS: Self = Self::Ok;
    const NULL: Self = Self::NullPassed;
    const PANIC: Self = Self::Panic;
}

// How to map an `Error` to an `MyFFIError`.
impl From<MyError> for MyFFIError {
    fn from(x: MyError) -> Self {
        match x {
            MyError::Bad => Self::OtherError,
        }
    }
}
#[ffi_type(opaque)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

// Helper function defining the type.
pub fn vec2_type() -> CType {
    let fields = vec![Field::new("x".to_string(), CType::Primitive(PrimitiveType::F32)),Field::new("y".to_string(), CType::Primitive(PrimitiveType::F32))];
    let composite = CompositeType::new("Vec2Type".to_string(), fields);
    CType::Composite(composite)
}

#[ffi_service(error = "MyFFIError", prefix = "rs_")]
impl Vec2 {
    #[ffi_service_ctor]
    pub fn new(x:f32,y:f32)->Result<Self,MyError> {
        Ok(Self {
            x,
            y
        })
    }
    #[ffi_service_method(on_panic = "return_default")]
    pub fn get_x(&self)->f32{
        self.x
    }
    #[ffi_service_method(on_panic = "return_default")]
    pub fn get_y(&self)->f32{
        self.y
    }



    #[ffi_service_method(on_panic = "undefined_behavior")]
    #[ffi_surrogates(x = "vec2_type")]
    pub fn add(&mut self,t:&Vec2){
        self.x+=t.x;
        self.y+=t.y;
    }
}


#[ffi_function]
#[ffi_surrogates(x = "vec2_type")]
#[no_mangle]
pub extern "C" fn my_function(input: Vec2) -> Vec2 {
    input
}

#[ffi_function]
#[no_mangle]
pub extern "C" fn callback<'a>(x:FFISlice<'a,u8>,input: extern "C" fn(FFISlice<'a,u8>)) {
    input(x);
}

#[ffi_function]
#[no_mangle]
pub extern "C" fn input(mut x:FFISliceMut<u8>) ->FFISliceMut<u8> {
    let z= x.deref_mut();
    z[0]=1;

    x
}

callback!(CallbackSlice(x: FFISlice<u8>) -> u8);
callback!(InputString(x:AsciiPointer));

#[ffi_function]
#[no_mangle]
pub extern "C" fn my_function3(callback: CallbackSlice) {
    let h="123123".to_string();
    callback.call(FFISlice::from_slice(h.as_bytes()));
}


#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn my_function4(callback: InputString) {
    let h="hello world\0".to_string();

    callback.call(AsciiPointer::from_cstr(CStr::from_bytes_with_nul(h.as_bytes()).unwrap()));
}



pub fn my_inventory() -> Inventory {
    InventoryBuilder::new()
        .register(function!(my_function))
        .register(function!(callback))
        .register(function!(input))
        .register(function!(my_function3))
        .register(function!(my_function4))
        .register(function!(my_api_guard))
        .register(function!(get_value))
        .register(constant!(SOME_CONST))
        .register(pattern!(Vec2))
        .inventory()
}