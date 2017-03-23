// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(Clone,Default)]
pub struct TextMessage {
    // message fields
    id: ::protobuf::SingularField<::std::string::String>,
    sender: ::protobuf::SingularField<::std::string::String>,
    text: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for TextMessage {}

impl TextMessage {
    pub fn new() -> TextMessage {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static TextMessage {
        static mut instance: ::protobuf::lazy::Lazy<TextMessage> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const TextMessage,
        };
        unsafe {
            instance.get(|| {
                TextMessage {
                    id: ::protobuf::SingularField::none(),
                    sender: ::protobuf::SingularField::none(),
                    text: ::protobuf::SingularField::none(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // required string id = 1;

    pub fn clear_id(&mut self) {
        self.id.clear();
    }

    pub fn has_id(&self) -> bool {
        self.id.is_some()
    }

    // Param is passed by value, moved
    pub fn set_id(&mut self, v: ::std::string::String) {
        self.id = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_id(&mut self) -> &mut ::std::string::String {
        if self.id.is_none() {
            self.id.set_default();
        };
        self.id.as_mut().unwrap()
    }

    // Take field
    pub fn take_id(&mut self) -> ::std::string::String {
        self.id.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_id(&self) -> &str {
        match self.id.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required string sender = 2;

    pub fn clear_sender(&mut self) {
        self.sender.clear();
    }

    pub fn has_sender(&self) -> bool {
        self.sender.is_some()
    }

    // Param is passed by value, moved
    pub fn set_sender(&mut self, v: ::std::string::String) {
        self.sender = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_sender(&mut self) -> &mut ::std::string::String {
        if self.sender.is_none() {
            self.sender.set_default();
        };
        self.sender.as_mut().unwrap()
    }

    // Take field
    pub fn take_sender(&mut self) -> ::std::string::String {
        self.sender.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_sender(&self) -> &str {
        match self.sender.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // required string text = 3;

    pub fn clear_text(&mut self) {
        self.text.clear();
    }

    pub fn has_text(&self) -> bool {
        self.text.is_some()
    }

    // Param is passed by value, moved
    pub fn set_text(&mut self, v: ::std::string::String) {
        self.text = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_text(&mut self) -> &mut ::std::string::String {
        if self.text.is_none() {
            self.text.set_default();
        };
        self.text.as_mut().unwrap()
    }

    // Take field
    pub fn take_text(&mut self) -> ::std::string::String {
        self.text.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_text(&self) -> &str {
        match self.text.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }
}

impl ::protobuf::Message for TextMessage {
    fn is_initialized(&self) -> bool {
        if self.id.is_none() {
            return false;
        };
        if self.sender.is_none() {
            return false;
        };
        if self.text.is_none() {
            return false;
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.id));
                },
                2 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.sender));
                },
                3 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.text));
                },
                _ => {
                    try!(::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields()));
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.id.iter() {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        for value in self.sender.iter() {
            my_size += ::protobuf::rt::string_size(2, &value);
        };
        for value in self.text.iter() {
            my_size += ::protobuf::rt::string_size(3, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.id.as_ref() {
            try!(os.write_string(1, &v));
        };
        if let Some(v) = self.sender.as_ref() {
            try!(os.write_string(2, &v));
        };
        if let Some(v) = self.text.as_ref() {
            try!(os.write_string(3, &v));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<TextMessage>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for TextMessage {
    fn new() -> TextMessage {
        TextMessage::new()
    }

    fn descriptor_static(_: ::std::option::Option<TextMessage>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "id",
                    TextMessage::has_id,
                    TextMessage::get_id,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "sender",
                    TextMessage::has_sender,
                    TextMessage::get_sender,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "text",
                    TextMessage::has_text,
                    TextMessage::get_text,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<TextMessage>(
                    "TextMessage",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for TextMessage {
    fn clear(&mut self) {
        self.clear_id();
        self.clear_sender();
        self.clear_text();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for TextMessage {
    fn eq(&self, other: &TextMessage) -> bool {
        self.id == other.id &&
        self.sender == other.sender &&
        self.text == other.text &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for TextMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

#[derive(Clone,Default)]
pub struct MessageAcknowledgement {
    // message fields
    message_id: ::protobuf::SingularField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for MessageAcknowledgement {}

impl MessageAcknowledgement {
    pub fn new() -> MessageAcknowledgement {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static MessageAcknowledgement {
        static mut instance: ::protobuf::lazy::Lazy<MessageAcknowledgement> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const MessageAcknowledgement,
        };
        unsafe {
            instance.get(|| {
                MessageAcknowledgement {
                    message_id: ::protobuf::SingularField::none(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // required string message_id = 1;

    pub fn clear_message_id(&mut self) {
        self.message_id.clear();
    }

    pub fn has_message_id(&self) -> bool {
        self.message_id.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message_id(&mut self, v: ::std::string::String) {
        self.message_id = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message_id(&mut self) -> &mut ::std::string::String {
        if self.message_id.is_none() {
            self.message_id.set_default();
        };
        self.message_id.as_mut().unwrap()
    }

    // Take field
    pub fn take_message_id(&mut self) -> ::std::string::String {
        self.message_id.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_message_id(&self) -> &str {
        match self.message_id.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }
}

impl ::protobuf::Message for MessageAcknowledgement {
    fn is_initialized(&self) -> bool {
        if self.message_id.is_none() {
            return false;
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.message_id));
                },
                _ => {
                    try!(::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields()));
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.message_id.iter() {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.message_id.as_ref() {
            try!(os.write_string(1, &v));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<MessageAcknowledgement>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for MessageAcknowledgement {
    fn new() -> MessageAcknowledgement {
        MessageAcknowledgement::new()
    }

    fn descriptor_static(_: ::std::option::Option<MessageAcknowledgement>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "message_id",
                    MessageAcknowledgement::has_message_id,
                    MessageAcknowledgement::get_message_id,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<MessageAcknowledgement>(
                    "MessageAcknowledgement",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for MessageAcknowledgement {
    fn clear(&mut self) {
        self.clear_message_id();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for MessageAcknowledgement {
    fn eq(&self, other: &MessageAcknowledgement) -> bool {
        self.message_id == other.message_id &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for MessageAcknowledgement {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

#[derive(Clone,Default)]
pub struct Envelope {
    // message fields
    message_type: ::std::option::Option<Envelope_Type>,
    recipient: ::protobuf::SingularField<::std::string::String>,
    text_message: ::protobuf::SingularPtrField<TextMessage>,
    message_acknowledgement: ::protobuf::SingularPtrField<MessageAcknowledgement>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Envelope {}

impl Envelope {
    pub fn new() -> Envelope {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Envelope {
        static mut instance: ::protobuf::lazy::Lazy<Envelope> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Envelope,
        };
        unsafe {
            instance.get(|| {
                Envelope {
                    message_type: ::std::option::Option::None,
                    recipient: ::protobuf::SingularField::none(),
                    text_message: ::protobuf::SingularPtrField::none(),
                    message_acknowledgement: ::protobuf::SingularPtrField::none(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // required .Envelope.Type message_type = 1;

    pub fn clear_message_type(&mut self) {
        self.message_type = ::std::option::Option::None;
    }

    pub fn has_message_type(&self) -> bool {
        self.message_type.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message_type(&mut self, v: Envelope_Type) {
        self.message_type = ::std::option::Option::Some(v);
    }

    pub fn get_message_type(&self) -> Envelope_Type {
        self.message_type.unwrap_or(Envelope_Type::TEXT_MESSAGE)
    }

    // required string recipient = 2;

    pub fn clear_recipient(&mut self) {
        self.recipient.clear();
    }

    pub fn has_recipient(&self) -> bool {
        self.recipient.is_some()
    }

    // Param is passed by value, moved
    pub fn set_recipient(&mut self, v: ::std::string::String) {
        self.recipient = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_recipient(&mut self) -> &mut ::std::string::String {
        if self.recipient.is_none() {
            self.recipient.set_default();
        };
        self.recipient.as_mut().unwrap()
    }

    // Take field
    pub fn take_recipient(&mut self) -> ::std::string::String {
        self.recipient.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_recipient(&self) -> &str {
        match self.recipient.as_ref() {
            Some(v) => &v,
            None => "",
        }
    }

    // optional .TextMessage text_message = 3;

    pub fn clear_text_message(&mut self) {
        self.text_message.clear();
    }

    pub fn has_text_message(&self) -> bool {
        self.text_message.is_some()
    }

    // Param is passed by value, moved
    pub fn set_text_message(&mut self, v: TextMessage) {
        self.text_message = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_text_message(&mut self) -> &mut TextMessage {
        if self.text_message.is_none() {
            self.text_message.set_default();
        };
        self.text_message.as_mut().unwrap()
    }

    // Take field
    pub fn take_text_message(&mut self) -> TextMessage {
        self.text_message.take().unwrap_or_else(|| TextMessage::new())
    }

    pub fn get_text_message(&self) -> &TextMessage {
        self.text_message.as_ref().unwrap_or_else(|| TextMessage::default_instance())
    }

    // optional .MessageAcknowledgement message_acknowledgement = 4;

    pub fn clear_message_acknowledgement(&mut self) {
        self.message_acknowledgement.clear();
    }

    pub fn has_message_acknowledgement(&self) -> bool {
        self.message_acknowledgement.is_some()
    }

    // Param is passed by value, moved
    pub fn set_message_acknowledgement(&mut self, v: MessageAcknowledgement) {
        self.message_acknowledgement = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message_acknowledgement(&mut self) -> &mut MessageAcknowledgement {
        if self.message_acknowledgement.is_none() {
            self.message_acknowledgement.set_default();
        };
        self.message_acknowledgement.as_mut().unwrap()
    }

    // Take field
    pub fn take_message_acknowledgement(&mut self) -> MessageAcknowledgement {
        self.message_acknowledgement.take().unwrap_or_else(|| MessageAcknowledgement::new())
    }

    pub fn get_message_acknowledgement(&self) -> &MessageAcknowledgement {
        self.message_acknowledgement.as_ref().unwrap_or_else(|| MessageAcknowledgement::default_instance())
    }
}

impl ::protobuf::Message for Envelope {
    fn is_initialized(&self) -> bool {
        if self.message_type.is_none() {
            return false;
        };
        if self.recipient.is_none() {
            return false;
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    };
                    let tmp = try!(is.read_enum());
                    self.message_type = ::std::option::Option::Some(tmp);
                },
                2 => {
                    try!(::protobuf::rt::read_singular_string_into(wire_type, is, &mut self.recipient));
                },
                3 => {
                    try!(::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.text_message));
                },
                4 => {
                    try!(::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.message_acknowledgement));
                },
                _ => {
                    try!(::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields()));
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.message_type.iter() {
            my_size += ::protobuf::rt::enum_size(1, *value);
        };
        for value in self.recipient.iter() {
            my_size += ::protobuf::rt::string_size(2, &value);
        };
        for value in self.text_message.iter() {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        for value in self.message_acknowledgement.iter() {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.message_type {
            try!(os.write_enum(1, v.value()));
        };
        if let Some(v) = self.recipient.as_ref() {
            try!(os.write_string(2, &v));
        };
        if let Some(v) = self.text_message.as_ref() {
            try!(os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited));
            try!(os.write_raw_varint32(v.get_cached_size()));
            try!(v.write_to_with_cached_sizes(os));
        };
        if let Some(v) = self.message_acknowledgement.as_ref() {
            try!(os.write_tag(4, ::protobuf::wire_format::WireTypeLengthDelimited));
            try!(os.write_raw_varint32(v.get_cached_size()));
            try!(v.write_to_with_cached_sizes(os));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn type_id(&self) -> ::std::any::TypeId {
        ::std::any::TypeId::of::<Envelope>()
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Envelope {
    fn new() -> Envelope {
        Envelope::new()
    }

    fn descriptor_static(_: ::std::option::Option<Envelope>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_enum_accessor(
                    "message_type",
                    Envelope::has_message_type,
                    Envelope::get_message_type,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "recipient",
                    Envelope::has_recipient,
                    Envelope::get_recipient,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_message_accessor(
                    "text_message",
                    Envelope::has_text_message,
                    Envelope::get_text_message,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_message_accessor(
                    "message_acknowledgement",
                    Envelope::has_message_acknowledgement,
                    Envelope::get_message_acknowledgement,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Envelope>(
                    "Envelope",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Envelope {
    fn clear(&mut self) {
        self.clear_message_type();
        self.clear_recipient();
        self.clear_text_message();
        self.clear_message_acknowledgement();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for Envelope {
    fn eq(&self, other: &Envelope) -> bool {
        self.message_type == other.message_type &&
        self.recipient == other.recipient &&
        self.text_message == other.text_message &&
        self.message_acknowledgement == other.message_acknowledgement &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Debug for Envelope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Envelope_Type {
    TEXT_MESSAGE = 1,
    MESSAGE_ACKNOWLEDGEMENT = 2,
}

impl ::protobuf::ProtobufEnum for Envelope_Type {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Envelope_Type> {
        match value {
            1 => ::std::option::Option::Some(Envelope_Type::TEXT_MESSAGE),
            2 => ::std::option::Option::Some(Envelope_Type::MESSAGE_ACKNOWLEDGEMENT),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [Envelope_Type] = &[
            Envelope_Type::TEXT_MESSAGE,
            Envelope_Type::MESSAGE_ACKNOWLEDGEMENT,
        ];
        values
    }

    fn enum_descriptor_static(_: Option<Envelope_Type>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("Envelope_Type", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for Envelope_Type {
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x23, 0x73, 0x72, 0x63, 0x2f, 0x63, 0x6c, 0x69, 0x65, 0x6e, 0x74, 0x2f, 0x6d, 0x65, 0x73,
    0x73, 0x61, 0x67, 0x65, 0x73, 0x2f, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x62, 0x75, 0x66, 0x73, 0x2e,
    0x70, 0x72, 0x6f, 0x74, 0x6f, 0x22, 0x49, 0x0a, 0x0b, 0x54, 0x65, 0x78, 0x74, 0x4d, 0x65, 0x73,
    0x73, 0x61, 0x67, 0x65, 0x12, 0x0e, 0x0a, 0x02, 0x69, 0x64, 0x18, 0x01, 0x20, 0x02, 0x28, 0x09,
    0x52, 0x02, 0x69, 0x64, 0x12, 0x16, 0x0a, 0x06, 0x73, 0x65, 0x6e, 0x64, 0x65, 0x72, 0x18, 0x02,
    0x20, 0x02, 0x28, 0x09, 0x52, 0x06, 0x73, 0x65, 0x6e, 0x64, 0x65, 0x72, 0x12, 0x12, 0x0a, 0x04,
    0x74, 0x65, 0x78, 0x74, 0x18, 0x03, 0x20, 0x02, 0x28, 0x09, 0x52, 0x04, 0x74, 0x65, 0x78, 0x74,
    0x22, 0x37, 0x0a, 0x16, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x41, 0x63, 0x6b, 0x6e, 0x6f,
    0x77, 0x6c, 0x65, 0x64, 0x67, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x12, 0x1d, 0x0a, 0x0a, 0x6d, 0x65,
    0x73, 0x73, 0x61, 0x67, 0x65, 0x5f, 0x69, 0x64, 0x18, 0x01, 0x20, 0x02, 0x28, 0x09, 0x52, 0x09,
    0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x49, 0x64, 0x22, 0x95, 0x02, 0x0a, 0x08, 0x45, 0x6e,
    0x76, 0x65, 0x6c, 0x6f, 0x70, 0x65, 0x12, 0x31, 0x0a, 0x0c, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67,
    0x65, 0x5f, 0x74, 0x79, 0x70, 0x65, 0x18, 0x01, 0x20, 0x02, 0x28, 0x0e, 0x32, 0x0e, 0x2e, 0x45,
    0x6e, 0x76, 0x65, 0x6c, 0x6f, 0x70, 0x65, 0x2e, 0x54, 0x79, 0x70, 0x65, 0x52, 0x0b, 0x6d, 0x65,
    0x73, 0x73, 0x61, 0x67, 0x65, 0x54, 0x79, 0x70, 0x65, 0x12, 0x1c, 0x0a, 0x09, 0x72, 0x65, 0x63,
    0x69, 0x70, 0x69, 0x65, 0x6e, 0x74, 0x18, 0x02, 0x20, 0x02, 0x28, 0x09, 0x52, 0x09, 0x72, 0x65,
    0x63, 0x69, 0x70, 0x69, 0x65, 0x6e, 0x74, 0x12, 0x2f, 0x0a, 0x0c, 0x74, 0x65, 0x78, 0x74, 0x5f,
    0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x18, 0x03, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x0c, 0x2e,
    0x54, 0x65, 0x78, 0x74, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x52, 0x0b, 0x74, 0x65, 0x78,
    0x74, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x12, 0x50, 0x0a, 0x17, 0x6d, 0x65, 0x73, 0x73,
    0x61, 0x67, 0x65, 0x5f, 0x61, 0x63, 0x6b, 0x6e, 0x6f, 0x77, 0x6c, 0x65, 0x64, 0x67, 0x65, 0x6d,
    0x65, 0x6e, 0x74, 0x18, 0x04, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x17, 0x2e, 0x4d, 0x65, 0x73, 0x73,
    0x61, 0x67, 0x65, 0x41, 0x63, 0x6b, 0x6e, 0x6f, 0x77, 0x6c, 0x65, 0x64, 0x67, 0x65, 0x6d, 0x65,
    0x6e, 0x74, 0x52, 0x16, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x41, 0x63, 0x6b, 0x6e, 0x6f,
    0x77, 0x6c, 0x65, 0x64, 0x67, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x22, 0x35, 0x0a, 0x04, 0x54, 0x79,
    0x70, 0x65, 0x12, 0x10, 0x0a, 0x0c, 0x54, 0x45, 0x58, 0x54, 0x5f, 0x4d, 0x45, 0x53, 0x53, 0x41,
    0x47, 0x45, 0x10, 0x01, 0x12, 0x1b, 0x0a, 0x17, 0x4d, 0x45, 0x53, 0x53, 0x41, 0x47, 0x45, 0x5f,
    0x41, 0x43, 0x4b, 0x4e, 0x4f, 0x57, 0x4c, 0x45, 0x44, 0x47, 0x45, 0x4d, 0x45, 0x4e, 0x54, 0x10,
    0x02, 0x4a, 0xf2, 0x05, 0x0a, 0x06, 0x12, 0x04, 0x00, 0x00, 0x14, 0x01, 0x0a, 0x0a, 0x0a, 0x02,
    0x04, 0x00, 0x12, 0x04, 0x00, 0x00, 0x04, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00, 0x01, 0x12,
    0x03, 0x00, 0x08, 0x13, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03, 0x01, 0x04,
    0x1b, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x04, 0x12, 0x03, 0x01, 0x04, 0x0c, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x05, 0x12, 0x03, 0x01, 0x0d, 0x13, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x01, 0x14, 0x16, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x01, 0x19, 0x1a, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02,
    0x01, 0x12, 0x03, 0x02, 0x04, 0x1f, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x04, 0x12,
    0x03, 0x02, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x05, 0x12, 0x03, 0x02,
    0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x02, 0x14, 0x1a,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03, 0x02, 0x1d, 0x1e, 0x0a, 0x0b,
    0x0a, 0x04, 0x04, 0x00, 0x02, 0x02, 0x12, 0x03, 0x03, 0x04, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x00, 0x02, 0x02, 0x04, 0x12, 0x03, 0x03, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02,
    0x02, 0x05, 0x12, 0x03, 0x03, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x01,
    0x12, 0x03, 0x03, 0x14, 0x18, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x03, 0x12, 0x03,
    0x03, 0x1b, 0x1c, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x01, 0x12, 0x04, 0x06, 0x00, 0x08, 0x01, 0x0a,
    0x0a, 0x0a, 0x03, 0x04, 0x01, 0x01, 0x12, 0x03, 0x06, 0x08, 0x1e, 0x0a, 0x0b, 0x0a, 0x04, 0x04,
    0x01, 0x02, 0x00, 0x12, 0x03, 0x07, 0x04, 0x23, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00,
    0x04, 0x12, 0x03, 0x07, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x05, 0x12,
    0x03, 0x07, 0x0d, 0x13, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x01, 0x12, 0x03, 0x07,
    0x14, 0x1e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x03, 0x12, 0x03, 0x07, 0x21, 0x22,
    0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x02, 0x12, 0x04, 0x0a, 0x00, 0x14, 0x01, 0x0a, 0x0a, 0x0a, 0x03,
    0x04, 0x02, 0x01, 0x12, 0x03, 0x0a, 0x08, 0x10, 0x0a, 0x0c, 0x0a, 0x04, 0x04, 0x02, 0x04, 0x00,
    0x12, 0x04, 0x0b, 0x04, 0x0e, 0x05, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x04, 0x00, 0x01, 0x12,
    0x03, 0x0b, 0x09, 0x0d, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03,
    0x0c, 0x08, 0x19, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03,
    0x0c, 0x08, 0x14, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x00, 0x02, 0x12, 0x03,
    0x0c, 0x17, 0x18, 0x0a, 0x0d, 0x0a, 0x06, 0x04, 0x02, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x0d,
    0x08, 0x24, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x0d,
    0x08, 0x1f, 0x0a, 0x0e, 0x0a, 0x07, 0x04, 0x02, 0x04, 0x00, 0x02, 0x01, 0x02, 0x12, 0x03, 0x0d,
    0x22, 0x23, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x00, 0x12, 0x03, 0x10, 0x04, 0x23, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x00, 0x04, 0x12, 0x03, 0x10, 0x04, 0x0c, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x02, 0x02, 0x00, 0x06, 0x12, 0x03, 0x10, 0x0d, 0x11, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x02, 0x02, 0x00, 0x01, 0x12, 0x03, 0x10, 0x12, 0x1e, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02,
    0x00, 0x03, 0x12, 0x03, 0x10, 0x21, 0x22, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x01, 0x12,
    0x03, 0x11, 0x04, 0x22, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x01, 0x04, 0x12, 0x03, 0x11,
    0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x01, 0x05, 0x12, 0x03, 0x11, 0x0d, 0x13,
    0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x01, 0x01, 0x12, 0x03, 0x11, 0x14, 0x1d, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x02, 0x02, 0x01, 0x03, 0x12, 0x03, 0x11, 0x20, 0x21, 0x0a, 0x0b, 0x0a, 0x04,
    0x04, 0x02, 0x02, 0x02, 0x12, 0x03, 0x12, 0x04, 0x2a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02,
    0x02, 0x04, 0x12, 0x03, 0x12, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x02, 0x06,
    0x12, 0x03, 0x12, 0x0d, 0x18, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x02, 0x01, 0x12, 0x03,
    0x12, 0x19, 0x25, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x02, 0x03, 0x12, 0x03, 0x12, 0x28,
    0x29, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x02, 0x02, 0x03, 0x12, 0x03, 0x13, 0x04, 0x40, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x02, 0x02, 0x03, 0x04, 0x12, 0x03, 0x13, 0x04, 0x0c, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x02, 0x02, 0x03, 0x06, 0x12, 0x03, 0x13, 0x0d, 0x23, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02,
    0x02, 0x03, 0x01, 0x12, 0x03, 0x13, 0x24, 0x3b, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x02, 0x02, 0x03,
    0x03, 0x12, 0x03, 0x13, 0x3e, 0x3f,
];

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
