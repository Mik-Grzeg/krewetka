import React from 'react';
import { Container } from 'react-bootstrap';
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
      <h3 className='text-center py-3 text-primary'>User Line Chart</h3>
        <ResponsiveContainer height={300} width='100%'>
          <LineChart width={600} height={300} data={data} margin={{ top: 5, right: 20, bottom: 5, left: 0 }}>
            <CartesianGrid stroke="#ccc" strokeDasharray="5 5" />
            <XAxis dataKey="time" />
            <Line type="monotone" dataKey="packets_per_second" stroke="#05f539" activeDot={{ r: 8 }} />
            <Legend />
            <Tooltip />
          </LineChart>
        </ResponsiveContainer>
      </Container>
    </div>
  );
}
