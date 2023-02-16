<template>
  <div class="w-full h-full flex flex-col justify-center items-center">
    <h1 class="text-4xl w-fit">Write Your Rust Code Here!</h1>
    <textarea @keydown="tabFix"
      class="bg-gray-700 rounded-md m-10 text-white w-[70%] p-3"
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

const code = ref("fn main() { \n\tprintln!(\"Hello, world!\"); \n}");
const term = ref("");

// const tabFix = (e: KeyboardEvent) => {
//   if (e.target != null && e.key === "Tab") {
//     e.preventDefault();
//     const start = e.target.selectionStart;
//     const end = e.target.selectionEnd;
//     code.value =
//       code.value.substring(0, start) +
//       "\t" +
//       code.value.substring(end, code.value.length);
//     e.target.selectionStart = e.target.selectionEnd = start + 1;
// };

const runCode = () => {
  console.log(code.value);
  const id = Math.random().toString(36).substring(10);

  fetch("http://127.0.0.1:7888/compile", {
    method: "POST",
    body: JSON.stringify({ id: id, code: code.value }),
  }).then((res) => {
    console.log(res);
  });

  const socket = new WebSocket("ws://127.0.0.1:7888", "chat");

  socket.onopen = function (e) {
    console.log("Connection established");
  };

  socket.onmessage = function (event) {
    console.log(`Data received from server: ${event.data}`);
    term.value += event.data;
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
};
</script>
