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

/*****************************************************************************/
/* Flows Metadata                                                            */
/*****************************************************************************/

header_type metadata_flows_t {
    fields {
        srcPort : 16;
        dstPort : 16;
        not_in_bloom_filter_1 : 1;
        not_in_bloom_filter_2 : 1;
        hash1 : HASH_WIDTH;
        hash2 : HASH_WIDTH;
    }
}

metadata metadata_flows_t md_flows_metadata;


/*****************************************************************************/
/* IPv4 Tuple                                                                */
/*****************************************************************************/

field_list ipv4_flows_tuple {
    ipv4.dstAddr;
    ipv4.srcAddr;
    ipv4.protocol;
    md_flows_metadata.srcPort;
    md_flows_metadata.dstPort;
}


/*****************************************************************************/
/* Extract Port Information                                                  */
/*****************************************************************************/

action get_flows_tcp_ports() {
    modify_field(md_flows_metadata.srcPort, tcp.srcPort);
    modify_field(md_flows_metadata.dstPort, tcp.dstPort);
}
action get_flows_udp_ports() {
    modify_field(md_flows_metadata.srcPort, udp.srcPort);
    modify_field(md_flows_metadata.dstPort, udp.dstPort);
}

table extract_flows_ports {
    reads {
        tcp : valid;
        udp : valid;
    }
    actions {
        get_flows_tcp_ports;
        get_flows_udp_ports;
        _nop;
    }
    size: 2;
}


/*****************************************************************************/
/* Copy Hashes                                                               */
/*****************************************************************************/

action copy_flows_hashes() {
    modify_field_with_hash_based_offset(md_flows_metadata.hash1, 0, flows_hash_1, TUPLE_FILTER_SIZE);
    modify_field_with_hash_based_offset(md_flows_metadata.hash2, 0, flows_hash_2, TUPLE_FILTER_SIZE);
}

table copy_flows_hashes {
    actions { copy_flows_hashes; }
}


/*****************************************************************************/
/* Forward Learned Flow                                                      */
/*****************************************************************************/

field_list ipv4_flows_tuple_plus_hash {
    ipv4.dstAddr;
    ipv4.srcAddr;
    ipv4.protocol;
    md_flows_metadata.srcPort;
    md_flows_metadata.dstPort;
    md_flows_metadata.hash1;
    md_flows_metadata.hash2;
}

action generate_flows_flow_learn() {
    generate_digest(FLOW_RECEIVER, ipv4_flows_tuple_plus_hash);
}

table learn_flows_flow {
    actions { generate_flows_flow_learn; }
}


/*****************************************************************************/
/* Bloom Filters                                                             */
/*****************************************************************************/

register flows_bloom_filter_1 {
    width          : 1;
    instance_count : TUPLE_FILTER_SIZE;
}

blackbox stateful_alu flows_check_and_learn_1 {
    reg                      : flows_bloom_filter_1;
    update_lo_1_value        : set_bitc;
    output_value             : alu_lo;
    output_dst               : md_flows_metadata.not_in_bloom_filter_1;
    initial_register_lo_value: 0;
}

field_list_calculation flows_hash_1 {
    input {ipv4_flows_tuple;}
    algorithm : crc16_extend;
    output_width : HASH_WIDTH;
}

action flows_check_and_learn_1() {
    flows_check_and_learn_1.execute_stateful_alu_from_hash(flows_hash_1);
}

table flows_bloom_filter_1 {
    actions { flows_check_and_learn_1; }
    default_action: flows_check_and_learn_1;
}


register flows_bloom_filter_2 {
    width          : 1;
    instance_count : TUPLE_FILTER_SIZE;
}

blackbox stateful_alu flows_check_and_learn_2 {
    reg                      : flows_bloom_filter_2;
    update_lo_1_value        : set_bitc;
    output_value             : alu_lo;
    output_dst               : md_flows_metadata.not_in_bloom_filter_2;
    initial_register_lo_value: 0;
}

field_list_calculation flows_hash_2 {
    input {ipv4_flows_tuple;}
    algorithm : crc32_msb;
    output_width : HASH_WIDTH;
}

action flows_check_and_learn_2() {
    flows_check_and_learn_2.execute_stateful_alu_from_hash(flows_hash_2);
}

table flows_bloom_filter_2 {
    actions { flows_check_and_learn_2; }
    default_action: flows_check_and_learn_2;
}



/*****************************************************************************/
/* Process Flows                                                             */
/*****************************************************************************/

control process_flows {
    apply(extract_flows_ports);
    apply(copy_flows_hashes);
    apply(flows_bloom_filter_1);
    apply(flows_bloom_filter_2);
    if (md_flows_metadata.not_in_bloom_filter_1 == 1 or
        md_flows_metadata.not_in_bloom_filter_2 == 1)
    {
        apply(learn_flows_flow);
    }
}
