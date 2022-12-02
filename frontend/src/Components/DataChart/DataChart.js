import React from 'react';
import { Container } from 'react-bootstrap';
import moment from 'moment';
import {
  LineChart,
  Line,
  XAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer
} from "recharts";

export default function DataChart({ data }) {
  return (
    <div>
      <Container>
      <h3 className='text-center py-3 text-primary'>Throughput stats</h3>
        <ResponsiveContainer height={400} width='100%'>
          <LineChart width={600} height={300} data={data} margin={{ top: 5, right: 20, bottom: 50, left: 0 }}>
            <CartesianGrid stroke="#ccc" strokeDasharray="5 5" />
						<XAxis dataKey="time" domain={['auto', 'auto']} tickFormatter={timeStr => moment(timeStr).format('MM/DD HH:mm')} angle={-45} textAnchor="end" />

            <Line type="monotone" dataKey="packets_per_second" stroke="#05f539" dot={false} activeDot={{stroke: 'red', strokeWidth: 2, r: 4}}/>
            <Legend />
            <Tooltip />
          </LineChart>
        </ResponsiveContainer>
      </Container>
    </div>
  );
}
