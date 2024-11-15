import {
    Chart,
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    CategoryScale,
    Title,
    Legend,
    Tooltip
} from 'chart.js';


Chart.register(
    LineController,
    LineElement,
    PointElement,
    LinearScale,
    CategoryScale,
    Title,
    Legend,
    Tooltip
);

export const chartInstances = {};

export function createChart(canvasId, data, options) {
    const chart = new Chart(
        document.getElementById(canvasId).getContext('2d'), {
        type: 'line', 
        data: data,
        options: options
    });

    chartInstances[canvasId] = chart;
    return chart;
}

export function updateChart(canvasId, newData) {
    const chart = chartInstances[canvasId];
    if (!chart) {
        console.error(`No chart found for canvasId: ${canvasId}.`);
        return;
    }   
    chart.data.labels = newData.labels; 
    chart.data.datasets.forEach((dataset, index) => {
        dataset.data = newData.datasets[index].data; 
    });

    chart.update();
}

