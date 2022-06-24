use std::mem::size_of;

use crate::{
    pdf::{
        trailer::TRAILER,
        xref::{UsedObject, XrefEntry, XrefKind},
        Array, Dictionary, IndirectObject, Name, Object, PdfSection, Stream, Trailer, Xref,
    },
    simple_encode::SimpleEncoder,
    writer::{Encoder, Writer},
};

impl Encoder<PdfSection> for SimpleEncoder {
    fn write_to(sec: &PdfSection, writer: &mut dyn Writer) {
        log::trace!("write PDF Section");

        // sort object keys
        let mut keys: Vec<usize> = sec.objects.keys().copied().collect();
        keys.sort_unstable();

        // prepare list of xref entries
        let mut xref_entries = Vec::<XrefEntry>::with_capacity(keys.len());

        // write objects and add XRef entry to list
        for &obj_index in keys.iter() {
            if let Some(obj) = sec.objects.get(&obj_index) {
                xref_entries.push(
                    UsedObject {
                        number: obj_index,
                        byte_offset: writer.position(),
                        generation: 0,
                    }
                    .into(),
                );
                Self::write_to(obj, writer);
            }
        }

        let start_xref = writer.position();
        Self::write_to(&Xref::from(xref_entries), writer);

        writer.write(b"startxref\n");
        writer.write(start_xref.to_string().as_bytes());
        writer.write(b"\n");
    }
}

fn xref_to_tuple(entry: &XrefEntry) -> (usize, usize, usize) {
    let xref_type = entry.type_num();
    match entry {
        XrefEntry::Free(entry) => (xref_type, entry.next_free, entry.generation),
        XrefEntry::Used(entry) => (xref_type, entry.byte_offset, entry.generation),
        XrefEntry::UsedCompressed(entry) => (xref_type, entry.containing_object, entry.index),
        XrefEntry::Unsupported(unsup) => (xref_type, unsup.w1, unsup.w2),
    }
}

impl Encoder<Xref> for SimpleEncoder {
    fn write_to(o: &Xref, writer: &mut dyn Writer) {
        log::trace!("write XRef");

        // Type-size = (1 byte we only know a few types), x-size, y-size
        // to keep it simple we just take the usize bytes and don't optimize here.
        let w_values = [1usize, size_of::<usize>(), size_of::<usize>()];

        let mut data = Vec::<u8>::with_capacity(o.len() * w_values.iter().sum::<usize>());
        for entry in o.entries() {
            encode_xref_entry(w_values, entry, &mut data);
        }

        let (index, generation) = if let Some(XrefKind::Stream { number, generation }) = o.kind {
            (number, generation)
        } else {
            // FIXME: no unwrap
            let index: u32 = o.highest_index().try_into().unwrap();
            (index + 1, 0)
        };

        let indirect_obj = Object::Indirect(IndirectObject {
            index,
            generation,
            object: Box::new(Object::Stream(Stream {
                dictionary: Dictionary::from([
                    (Name::from_str("Type"), Object::from(Name::from_str("XRef"))),
                    (
                        Name::from_str("Size"),
                        Object::from(i32::try_from(o.highest_index()).unwrap() + 1),
                    ),
                    (
                        Name::from_str("W"),
                        Object::from(Array::from(
                            [
                                Object::from(i32::try_from(w_values[0]).unwrap()),
                                Object::from(i32::try_from(w_values[1]).unwrap()),
                                Object::from(i32::try_from(w_values[2]).unwrap()),
                            ]
                            .to_vec(),
                        )),
                    ),
                ]),
                data: data.into(),
            })),
        });

        Self::write_to(&indirect_obj, writer);
    }
}

fn encode_xref_entry(w_length: [usize; 3], entry: &XrefEntry, buffer: &mut Vec<u8>) {
    let (w1, w2, w3) = xref_to_tuple(entry);
    let w1 = w1.to_be_bytes();
    let w2 = w2.to_be_bytes();
    let w3 = w3.to_be_bytes();

    // Make sure that we don't use more space than allocated.
    // FIXME: If the specified length (w_length) is bigger than the w1,w2,w3 arrays
    // the resulting array is offset and invalid.
    buffer.extend_from_slice(&w1[..w_length[0].min(size_of::<usize>())]);
    buffer.extend_from_slice(&w2[..w_length[1].min(size_of::<usize>())]);
    buffer.extend_from_slice(&w3[..w_length[2].min(size_of::<usize>())]);
}

impl Encoder<Trailer> for SimpleEncoder {
    fn write_to(trailer: &Trailer, writer: &mut dyn Writer) {
        log::trace!("write Trailer");

        let trailer_dict: Dictionary = trailer.clone().into();
        writer.write(TRAILER);
        writer.write(b"\n");
        Self::write_to(&trailer_dict, writer);
        writer.write(b"\n");
    }
}
