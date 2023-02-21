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
      :disabled="loading"
      @click="runCode"
    >
      Compile And Run
    </button>
    <div class="border-gray-700 border-solid border-2 rounded-md w-[70%] h-auto m-10 p-3">
      <ul>
        <li v-for="line in term" :key="line"><div v-html="line"></div></li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Ref, ref } from "vue";
import ath from "ansi-to-html"
import NProgress from "nprogress";

let convert = new ath; // Convert ANSI to HTML

const code = ref("fn main() { \n\tprintln!(\"Hello, world!\"); \n}");
const term: Ref<string[]> = ref([]);
const loading = ref(false);

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

const genRand = (len: number) => {
  return Math.random().toString(36).substring(2,len+2);
}

const runCode = () => {
  term.value = [];
  loading.value = true;
  NProgress.start();
  const id = genRand(8);
  console.log(id, code.value);

  fetch("http://127.0.0.1:7888/compile", {
    method: "POST",
    body: JSON.stringify({ id: id, code: code.value }),
  }).then((res) => {
    if (res.status == 200) {
      connectToSocket(id);
    } else {
      alert("Error");
      NProgress.done();

    }
  });
};

const connectToSocket = (id: string) => {
  console.log("Connecting to socket");
  const socket = new WebSocket("ws://127.0.0.1:7888", "chat");

  socket.onopen = function (e) {
    console.log("Connection established");
    socket.send(id);
  };

  socket.onmessage = function (event) {
    NProgress.done()
    console.log(`Data received from server: ${event.data}`);
    term.value.push(convert.toHtml(event.data));
  };

  socket.onclose = function (event) {
    if (event.wasClean) {
      console.log(
        `Connection closed cleanly, code=${event.code} reason=${event.reason}`
      );
    } else {
      console.log("Connection died");
      loading.value = false;
    }
  };

  socket.onerror = function (error) {
    console.log("error", error);
  };
};
</script>
