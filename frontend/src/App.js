import { React, useEffect, useState } from 'react';
import PieCharts from './Components/PieCharts/PieCharts';
import TableData from './Components/TableData/TableData';
import DataChart from './Components/DataChart/DataChart';
import 'bootstrap/dist/css/bootstrap.min.css';
import { Col, Container, Row } from 'react-bootstrap';

function App() {
	const [data, setData] = useState([]);

	useEffect(() => {
		const ws = new WebSocket("ws://localhost:8080/throughput_ws");
		ws.onopen = (event) => {
			console.log(JSON.stringify(event))
			console.log("[open] Connection established")
			console.log("Sending to server")
		};

		ws.onmessage = function (event) {
			const json = JSON.parse(event.data);
			try {
				if ((json.event = "data")) {
					json.forEach(elem => {
						elem.time = new Date(elem.time).toLocaleString()
					})
					setData(oldData => [...oldData, ...json]);
				}
			} catch (err) {
				console.log(err);
			}
		};

		return () => ws.close();
	}, []);

  return (
    <div>
      <Container>
        <Row className='gy-5'>

          <Col xs={12} md={12} lg={12}>
            <DataChart data={data}/>
          </Col>
          <Col xs={12} md={12} lg={12}><h3 className='text-center text-primary'>Data show in Pie chart</h3>
            <PieCharts></PieCharts></Col>
          <Col xs={12} md={12} lg={12}>
            <TableData></TableData>
          </Col>
        </Row>
      </Container>
    </div >
  );
}

export default App;
