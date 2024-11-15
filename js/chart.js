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

// Register necessary components
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

// Keep track of active chart instances
export const chartInstances = {};

export function createChart(canvasId, data, options) {
    const canvas = document.getElementById(canvasId);
    const ctx = canvas.getContext('2d');
    const chart = new Chart(ctx, {
        type: 'line', // Example chart type
        data: data,
        options: options
    });

    chartInstances[canvasId] = chart;
    return chart;
}

// Assuming chartInstances and createChart are already defined as in previous examples

export function updateChart(canvasId, newData) {
    const chart = chartInstances[canvasId];

    if (!chart) {
        console.error(`No chart found for canvasId: ${canvasId}.`);
        return;
    }

    // Update chart data
    chart.data.labels = newData.labels; // Update labels
    chart.data.datasets.forEach((dataset, index) => {
        dataset.data = newData.datasets[index].data; // Update dataset data
    });

    // Apply the updates
    chart.update();
}

