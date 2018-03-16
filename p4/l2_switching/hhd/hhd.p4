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
/* Packet Counters for individual Hashes                                     */
/*****************************************************************************/

/* Counter Hash 1 */
counter count_hhd_hash_1 {
    type: packets;
    instance_count: TUPLE_FILTER_SIZE;
}

action count_hhd_packets_1() {
  count(count_hhd_hash_1, md_flows_metadata.hash1);
}

@pragma force_table_dependency copy_flows_hashes
table counter_hhd_hashes_1 {
  actions { count_hhd_packets_1; }
  size : TUPLE_FILTER_SIZE;
}

/* Counter Hash 2 */
counter count_hhd_hash_2 {
    type: packets;
    instance_count: TUPLE_FILTER_SIZE;
}

action count_hhd_packets_2() {
    count(count_hhd_hash_2, md_flows_metadata.hash2);
}

@pragma force_table_dependency copy_flows_hashes
table counter_hhd_hashes_2 {
    actions { count_hhd_packets_2; }
    size : TUPLE_FILTER_SIZE;
}


/*****************************************************************************/
/* Process HHD                                                               */
/*****************************************************************************/

control process_hhd {
    apply(counter_hhd_hashes_1);
    apply(counter_hhd_hashes_2);
}
