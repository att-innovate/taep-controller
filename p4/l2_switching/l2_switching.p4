// The MIT License (MIT)
//
// Copyright (c) 2018 AT&T. All other rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

#include <tofino/constants.p4>
#include <tofino/intrinsic_metadata.p4>
#include <tofino/primitives.p4>
#include <tofino/pktgen_headers.p4>
#include <tofino/stateful_alu_blackbox.p4>
#include <tofino/wred_blackbox.p4>
#include "includes/defines.p4"
#include "includes/headers.p4"
#include "includes/parser.p4"
#include "flows/flows.p4"
#include "hhd/hhd.p4"


/*****************************************************************************/
/* Forward Packet                                                            */
/*****************************************************************************/

action set_egr(egress_spec) {
    modify_field(ig_intr_md_for_tm.ucast_egress_port, egress_spec);
}

table forward {
    reads {
        ig_intr_md.ingress_port : exact;
    }
    actions {
        set_egr;
        _nop;
    }
    size: BAREFOOT_MAX_PORTS;
}

/*****************************************************************************/
/* Divert Packet                                                             */
/*****************************************************************************/

table divert {
    reads {
        ig_intr_md.ingress_port : exact;
        ipv4.dstAddr : ternary;
        ipv4.srcAddr : ternary;
    }
    actions {
        set_egr;
        _nop;
    }
    size: BAREFOOT_MAX_PORTS;
}

/*****************************************************************************/
/* Enable/Disable Feature per Port                                           */
/*****************************************************************************/

header_type metadata_feature_t {
    fields {
        hhd : 1;
        flows : 1;
    }
}

metadata metadata_feature_t feature_metadata;

action feature_enable(hhd, flows) {
    modify_field(feature_metadata.hhd, hhd);
    modify_field(feature_metadata.flows, flows);
}

table feature {
    reads {
        ig_intr_md.ingress_port : exact;
    }
    actions {
        feature_enable;
    }
    size: BAREFOOT_MAX_PORTS;
}

/*****************************************************************************/
/* Ingress/Egress Control                                                    */
/*****************************************************************************/

control ingress {
    // force to be in phase 0
    if (ig_intr_md.resubmit_flag == 0) {
        apply(feature);
    }

    apply(forward);
    apply(divert);

    if ((feature_metadata.hhd == TRUE) or (feature_metadata.flows == TRUE))
        process_flows();

    if (feature_metadata.hhd == TRUE)
        process_hhd();

}

control egress {
}
