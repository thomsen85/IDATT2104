<template>
  <div class="w-full h-full flex flex-col justify-center items-center">
    <h1 class="text-4xl w-fit">Write Your Rust Code Here!</h1>
    <textarea
      class="bg-gray-700 rounded-md m-10 text-white w-[70%]"
      id="code"
      cols="30"
      rows="10"
      v-model="code"
    ></textarea>
    <button
      class="bg-blue-500 rounded-md w-fit p-3 text-white"
      @click="runCode"
    >
      Compile And Run
    </button>
    <p>
      {{ term }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

const code = ref("");
const term = ref("");

const runCode = () => {
  console.log(code.value);
};

const socket = new WebSocket("ws://127.0.0.1:7888", "chat");

socket.onopen = function (e) {
  console.log("Connection established");
  // setInterval(function () {
  //   if (socket.bufferedAmount == 0) {
  //     console.log('Sending Message to server');
  //     socket.send('thomas');
  //   }
  // }, 500);
};

socket.onmessage = function (event) {
  alert(`Data received from server: ${event.data}`);
  console.log(
    `Data received from server: ${event.data}`
    );
};

socket.onclose = function (event) {
  if (event.wasClean) {
    console.log(
      `Connection closed cleanly, code=${event.code} reason=${event.reason}`
    );
  } else {
    console.log("Connection died");
  }
};

socket.onerror = function (error) {
  console.log("error", error);
};
</script>
