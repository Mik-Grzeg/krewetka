import { React, useEffect, useState } from 'react';
import PieCharts from './Components/PieCharts/PieCharts';
import TableData from './Components/TableData/TableData';
import DataChart from './Components/DataChart/DataChart';
import 'bootstrap/dist/css/bootstrap.min.css';
import { Col, Container, Row } from 'react-bootstrap';

const fillWholeInterval = (start, aggr_interval) => {
	let coeff = 1000 * aggr_interval
  let now = new Date()
	var now_t = new Date(Math.floor(now.getTime() / coeff) * coeff)
	var result = {}

	for (let i = start.getTime(); i < now_t; i+=aggr_interval) {
		result[new Date(i).toISOString()] = 0
	}

	return result
}

function App() {
	const [data, setData] = useState([]);

	useEffect(() => {
		var start_time = new Date()
		start_time.setMinutes(start_time.getMinutes() - 15);
		var aggr_interval = 60;

		const ws = new WebSocket(`ws://krewetka.norwayeast.cloudapp.azure.com/analytics/api/v1/throughput?start_period=${encodeURIComponent(start_time.toISOString())}&aggr_interval=${aggr_interval}`);
		ws.onopen = (event) => {
			console.log(JSON.stringify(event))
			console.log("[open] Connection established")
		};

		ws.onmessage = function (event) {
			const json = JSON.parse(event.data);
			try {
				if ((json.event = "data")) {
					var coeff = 1000 * aggr_interval;
					// var date = new Date();  //or use any other date
					var roundedStart = new Date(Math.floor(start_time.getTime() / coeff) * coeff)

					var baseDates = fillWholeInterval(roundedStart, aggr_interval * 1000)

					json.forEach(elem => {
						baseDates[new Date(elem.time).toISOString()] = elem.packets_per_second
					})

					var result = Object.keys(baseDates).map(k => {
						return {
							time: k,
							packets_per_second: baseDates[k]
						}
					})

					setData(oldData => [...oldData, ...result]);
					start_time.setDate(roundedStart)
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
