use neli::{
    attr::Attribute,
    consts::{genl::*, nl::*, socket::*},
    err::NlError,
    genl::Genlmsghdr,
    nl::{NlPayload, Nlmsghdr},
    socket::NlSocketHandle,
    types::{Buffer, GenlBuffer},
};

const GENL_VERSION: u8 = 2;

// This example attempts to mimic the "genl ctrl list" command. For simplicity, it only outputs
// the name and identifier of each generic netlink family.

fn main() -> Result<(), NlError> {
    let mut socket = NlSocketHandle::connect(NlFamily::Generic, None, &[])?;

    let attrs = GenlBuffer::<NlAttrTypeWrapper, Buffer>::new();
    let genlhdr = Genlmsghdr::new(CtrlCmd::Getfamily, GENL_VERSION, attrs);
    let nlhdr = {
        let len = None;
        let nl_type = GenlId::Ctrl;
        let flags = NlmFFlags::new(&[NlmF::Request, NlmF::Dump]);
        let seq = None;
        let pid = None;
        Nlmsghdr::new(len, nl_type, flags, seq, pid, NlPayload::Payload(genlhdr))
    };
    socket.send(nlhdr)?;

    let iter = socket.iter::<Genlmsghdr<CtrlCmd, CtrlAttr>>(false);
    for response_result in iter {
        let response = response_result?;

        // FIXME: This example could be improved by
        // reinterpreting the payload as an Nlmsgerr struct
        // and printing the specific error encountered.
        if let NlTypeWrapper::Nlmsg(Nlmsg::Error) = response.nl_type {
            return Err(NlError::new(
                "An error occurred while retrieving available families",
            ));
        }

        let handle = response.get_payload()?.get_attr_handle();

        for attr in handle.iter() {
            match &attr.nla_type {
                CtrlAttr::FamilyName => {
                    println!("{}", attr.get_payload_as::<String>()?);
                }
                CtrlAttr::FamilyId => {
                    println!("\tID: 0x{:x}", attr.get_payload_as::<u16>()?);
                }
                _ => (),
            }
        }
    }

    Ok(())
}
