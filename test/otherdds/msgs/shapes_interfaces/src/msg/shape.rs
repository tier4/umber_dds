use safe_drive::{msg::TypeSupport, rcl::{self, size_t}};

#[repr(C)]
#[derive(Debug)]
pub struct Shape {
    pub color: safe_drive::msg::RosString<0>,
    pub x: i32,
    pub y: i32,
    pub shape_size: i32,
}

extern "C" {
    fn shapes_interfaces__msg__Shape__init(msg: *mut Shape) -> bool;
    fn shapes_interfaces__msg__Shape__fini(msg: *mut Shape);
    fn shapes_interfaces__msg__Shape__are_equal(lhs: *const Shape, rhs: *const Shape) -> bool;
    fn shapes_interfaces__msg__Shape__Sequence__init(msg: *mut ShapeSeqRaw, size: usize) -> bool;
    fn shapes_interfaces__msg__Shape__Sequence__fini(msg: *mut ShapeSeqRaw);
    fn shapes_interfaces__msg__Shape__Sequence__are_equal(lhs: *const ShapeSeqRaw, rhs: *const ShapeSeqRaw) -> bool;
    fn rosidl_typesupport_c__get_message_type_support_handle__shapes_interfaces__msg__Shape() -> *const rcl::rosidl_message_type_support_t;
}

impl TypeSupport for Shape {
    fn type_support() -> *const rcl::rosidl_message_type_support_t {
        unsafe {
            rosidl_typesupport_c__get_message_type_support_handle__shapes_interfaces__msg__Shape()
        }
    }
}

impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            shapes_interfaces__msg__Shape__are_equal(self, other)
        }
    }
}

impl<const N: usize> PartialEq for ShapeSeq<N> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let msg1 = ShapeSeqRaw{ data: self.data, size: self.size, capacity: self.capacity };
            let msg2 = ShapeSeqRaw{ data: other.data, size: other.size, capacity: other.capacity };
            shapes_interfaces__msg__Shape__Sequence__are_equal(&msg1, &msg2)
        }
    }
}

impl Shape {
    pub fn new() -> Option<Self> {
        let mut msg: Self = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        if unsafe { shapes_interfaces__msg__Shape__init(&mut msg) } {
            Some(msg)
        } else {
            None
        }
    }
}

impl Drop for Shape {
    fn drop(&mut self) {
        unsafe { shapes_interfaces__msg__Shape__fini(self) };
    }
}

#[repr(C)]
#[derive(Debug)]
struct ShapeSeqRaw {
    data: *mut Shape,
    size: size_t,
    capacity: size_t,
}

/// Sequence of Shape.
/// `N` is the maximum number of elements.
/// If `N` is `0`, the size is unlimited.
#[repr(C)]
#[derive(Debug)]
pub struct ShapeSeq<const N: usize> {
    data: *mut Shape,
    size: size_t,
    capacity: size_t,
}

impl<const N: usize> ShapeSeq<N> {
    /// Create a sequence of.
    /// `N` represents the maximum number of elements.
    /// If `N` is `0`, the sequence is unlimited.
    pub fn new(size: usize) -> Option<Self> {
        if N != 0 && size > N {
            // the size exceeds in the maximum number
            return None;
        }
        let mut msg: ShapeSeqRaw = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        if unsafe { shapes_interfaces__msg__Shape__Sequence__init(&mut msg, size) } {
            Some(Self { data: msg.data, size: msg.size, capacity: msg.capacity })
        } else {
            None
        }
    }

    pub fn null() -> Self {
        let msg: ShapeSeqRaw = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
        Self { data: msg.data, size: msg.size, capacity: msg.capacity }
    }

    pub fn as_slice(&self) -> &[Shape] {
        if self.data.is_null() {
            &[]
        } else {
            let s = unsafe { std::slice::from_raw_parts(self.data, self.size as _) };
            s
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [Shape] {
        if self.data.is_null() {
            &mut []
        } else {
            let s = unsafe { std::slice::from_raw_parts_mut(self.data, self.size as _) };
            s
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Shape> {
        self.as_slice().iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Shape> {
        self.as_slice_mut().iter_mut()
    }

    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<const N: usize> Drop for ShapeSeq<N> {
    fn drop(&mut self) {
        let mut msg = ShapeSeqRaw{ data: self.data, size: self.size, capacity: self.capacity };
        unsafe { shapes_interfaces__msg__Shape__Sequence__fini(&mut msg) };
    }
}

unsafe impl<const N: usize> Send for ShapeSeq<N> {}
unsafe impl<const N: usize> Sync for ShapeSeq<N> {}
