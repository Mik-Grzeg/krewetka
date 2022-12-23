import { React, useEffect, useState } from "react";
import { Spinner, Table } from "react-bootstrap";
import ModalData from "./ModalData/ModalData";
import "./TableData.css";

const TableData = () => {
  const [data, setData] = useState([]);
  const [showAll, setShowAll] = useState({});
  const [show, setShow] = useState(false);

  const handleClose = () => setShow(false);
  const handleShow = () => setShow(true);

  useEffect(() => {
    var start_time = new Date();
    start_time.setMinutes(start_time.getMinutes() - 15);

    var host = "pingu-5.16.9-gentoo-x86_64";

    const ws = new WebSocket(
      `ws://krewetka.norwayeast.cloudapp.azure.com/analytics/api/v1/flows_details?host=${encodeURIComponent(
        host
      )}&start_period=${encodeURIComponent(start_time.toISOString())}`
    );
    ws.onopen = (event) => {
      console.log(JSON.stringify(event));
      console.log("[open] Connection established");
    };

    ws.onmessage = function (event) {
      const fetchedData = JSON.parse(event.data);
      try {
        if ((fetchedData.event = "data")) {
          setData([...data, ...fetchedData]);
        }
      } catch (err) {
        console.log(err);
      }
    };

    return () => ws.close();
  }, []);

  console.log(data);
  const handleDetails = (ip) => {
    const selectedRow = data.find((d) => d.ipv4_dst_addr === ip);
    setShowAll(selectedRow);
    handleShow();
  };
  return (
    <div className="table-height-style">
      <Table bordered hover responsive>
        <thead>
          <tr>
            <th scope="col">ip in</th>
            <th scope="col">port in</th>
            <th scope="col">ip out</th>
            <th scope="col">port out</th>
            <th scope="col">protocol</th>
            <th scope="col">timestamp</th>
            <th scope="col">host</th>
          </tr>
        </thead>
        <tbody>
          {data.length === 0 ? (
            <div className="d-flex justify-content-center">
              <Spinner className="mt-5" animation="border" role="status">
                <span className="visually-hidden">Loading...</span>
              </Spinner>
            </div>
          ) : (
            <>
              {data?.slice(data.length - 101, data.length - 1).map((d, i) => (
                <tr
                  key={i}
                  style={{ cursor: "pointer" }}
                  onClick={() => handleDetails(d.ipv4_dst_addr)}
                >
                  <td>{d.ipv4_dst_addr}</td>
                  <td>{d.l4_dst_port}</td>
                  <td>{d.ipv4_src_addr}</td>
                  <td>{d.l4_src_port}</td>
                  <td>{d.protocol}</td>
                  <td>{d.timestamp}</td>
                  <td>{d.host}</td>
                </tr>
              ))}
            </>
          )}
        </tbody>
      </Table>
      <ModalData
        showAll={showAll}
        handleClose={handleClose}
        show={show}
      ></ModalData>
    </div>
  );
};

export default TableData;
