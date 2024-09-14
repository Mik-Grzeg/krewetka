import { React, useEffect } from 'react';
import "./PieCharts.css";
import { useCallback, useState } from "react";
import { PieChart, ResponsiveContainer, Cell, Pie, Sector } from "recharts";
import { Container } from 'react-bootstrap';

const COLORS = ['#0f4b7c', '#a1c4ec', '#dddddd'];

const renderActiveShape = (props) => {
  const RADIAN = Math.PI / 180;
  const {
    cx,
    cy,
    midAngle,
    innerRadius,
    outerRadius,
    startAngle,
    endAngle,
    fill,
    payload,
    percent,
    value,
    name
  } = props;
  const sin = Math.sin(-RADIAN * midAngle);
  const cos = Math.cos(-RADIAN * midAngle);
  const sx = cx + (outerRadius + 10) * cos;
  const sy = cy + (outerRadius + 10) * sin;
  const mx = cx + (outerRadius + 30) * cos;
  const my = cy + (outerRadius + 30) * sin;
  const ex = mx + (cos >= 0 ? 1 : -1) * 22;
  const ey = my;
  const textAnchor = cos >= 0 ? "start" : "end";
  return (
    <g>
      <text x={cx} y={cy} dy={8} textAnchor="middle" fill={fill}>
        {payload.name}
      </text>
      <Sector
        cx={cx}
        cy={cy}
        innerRadius={innerRadius}
        outerRadius={outerRadius}
        startAngle={startAngle}
        endAngle={endAngle}
        fill={fill}
      />
      <path
        d={`M${sx},${sy}L${mx},${my}L${ex},${ey}`}
        stroke={fill}
        fill="none"
      />
      <circle cx={ex} cy={ey} r={2} fill={fill} stroke="none" />
      <text
        x={ex + (cos >= 0 ? 1 : -1) * 12}
        y={ey}
        textAnchor={textAnchor}
        fill="#333"
      >{`${name} : ${value}`}</text>
      <text
        x={ex + (cos >= 0 ? 1 : -1) * 12}
        y={ey}
        dy={18}
        textAnchor={textAnchor}
        fill="#999"
      >
        {`(Rate ${(percent * 100).toFixed(2)}%)`}
      </text>
    </g>
  );
};

function PieCharts() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [data, setData] = useState();
  const onPieEnter = useCallback(
    (_, index) => {
      setActiveIndex(index);
    },
    [setActiveIndex]
  );


  useEffect( () => {
    let isMounted = true;

    function get_data() {
      return fetch("http://rest.norwayeast.cloudapp.azure.com/analytics/api/v1/grouped_packets_number")
      .then(res => res.json())
      .then(
        (result) => {
          var fetched = Object.keys(result).map(key => ({
            name: key,
            value: result[key]
          }));

          console.log("fetched data: ", JSON.stringify(fetched));
          console.log("type of data", typeof(fetched));
          if (isMounted) {
            setData(fetched);
          }
        }
      )
    }

    get_data();
    return () => {
      isMounted = false;
    };
  }, []);

  // const data = get_data();
  // get_data();
  return (
    <div>
      <Container>
      <ResponsiveContainer width="100%" height={400}>
      <PieChart width={400} height={400}>
          <Pie
            activeIndex={activeIndex}
            activeShape={renderActiveShape}
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={110}
            outerRadius={150}
            fill="#8884d8"
            dataKey="value"
            onMouseEnter={onPieEnter}>
            {data?.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
            ))}
          </Pie>
        </PieChart>
        </ResponsiveContainer>
      </Container>
    </div>
  );
}
export default PieCharts;
