<script setup lang="ts">
import { onMounted, ref } from 'vue';

const connected = ref(false);
const canvas = ref<HTMLCanvasElement | null>(null);
let ctx: CanvasRenderingContext2D | null = null;
let socket: WebSocket | null = null;

onMounted(() => {
  console.log(canvas.value);
  if (canvas.value) {
    ctx = canvas.value.getContext("2d");
  }
});

const connectToSocket = () => {
  console.log("Connecting to socket");
  socket = new WebSocket("ws://localhost:7888", "chat");

  socket.onopen = function (e) {
    console.log("Connection established");
    connected.value = true;
  };

  socket.onmessage = function (event) {
    let data_p = event.data.slice(0, -1);
    let data = JSON.parse(data_p);
    drawLine(data.x1, data.y1, data.x2, data.y2, false);
  };

  socket.onclose = function (event) {
    connected.value = false;
    if (event.wasClean) {
      console.log(
        `Connection closed cleanly, code=${event.code} reason=${event.reason}`
      );
    } else {
      console.log("Connection died");
    }
  };

  socket.onerror = function (error) {
    connected.value = false;
    console.log("error", error);
  };
};

const drawLine = (x1: number, y1: number, x2: number, y2: number, local: boolean) => {
  if (ctx) {
    ctx.beginPath();
    ctx.strokeStyle = 'black';
    ctx.lineWidth = 1;
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
    ctx.closePath();
    if (local) {
      socket?.send(JSON.stringify({ x1, y1, x2, y2 }));
    }
  }
}

const x = ref(0);
const y = ref(0);
const isDrawing = ref(false);

const beginDrawing = (e: any) => {
  x.value = e.offsetX;
  y.value = e.offsetY;
  isDrawing.value = true;
}
const keepDrawing = (e: any) => {
  if (isDrawing.value) {
    drawLine(x.value, y.value, e.offsetX, e.offsetY, true);
    x.value = e.offsetX;
    y.value = e.offsetY;
  }
}
const stopDrawing = (e: any) => {
  if (isDrawing.value) {
    drawLine(x.value, y.value, e.offsetX, e.offsetY, true);
    x.value = 0;
    y.value = 0;
    isDrawing.value = false;
  }
}
  

</script>

<template>
<h1>Websocket Drawer</h1>
<div class="container">
  <button v-if="!connected" @click="connectToSocket()">Connect</button>
  <canvas ref="canvas" width="500" height="500" @mousedown="beginDrawing" @mousemove="keepDrawing" @mouseup="stopDrawing"></canvas>
</div>
</template>

<style scoped>
canvas {
  border: 1px solid grey;
}
</style>
