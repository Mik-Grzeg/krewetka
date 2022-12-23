import React from "react";
import { Col, Modal, Row } from "react-bootstrap";
import '../TableData.css'
import {FaTimes} from 'react-icons/fa'
const ModalData = ({ show, handleClose, showAll }) => {
  const {
    host,
    flow_duration_milliseconds,
    in_bytes,
    in_pkts,
    ipv4_dst_addr,
    ipv4_src_addr,
    l4_dst_port,
    l4_src_port,
    l7_proto,
    malicious,
    out_bytes,
    out_pkts,
    protocol,
    tcp_flags,
    timestamp,
  } = showAll;
  return (
    <div>
      <Modal size="lg" show={show} onHide={handleClose}>
        <div className="modal-style">
          <p className="text-center fw-bold">{host}</p>
            <Row>
              <Col xs={16} md={6} lg={6}>
                <p>Flow Duration Milliseconds: {flow_duration_milliseconds}</p>
                <p>in_bytes: {in_bytes}</p>
                <p>in_pkts: {in_pkts}</p>
                <p>ipv4_dst_addr: {ipv4_dst_addr}</p>
                <p> ipv4_src_addr: {ipv4_src_addr}</p>
                <p>l4_dst_port: {l4_dst_port}</p>
                <p>l4_src_port: {l4_src_port}</p>
                <p>l7_proto: {l7_proto}</p>
              </Col>
              <Col xs={16} md={6} lg={6}>

                <p>malicious: {malicious}</p>
                <p>out_bytes: {out_bytes}</p>
                <p>out_pkts: {out_pkts}</p>
                <p>protocol: {protocol}</p>
                <p>tcp_flags: {tcp_flags}</p>
                <p>timestamp: {timestamp}</p>
              </Col>
            </Row>
            <button onClick={handleClose} className="btn-style">Okay</button>
            <FaTimes onClick={handleClose} className="close-btn" />
          </div>

      </Modal>
    </div>
  );
};

export default ModalData;
