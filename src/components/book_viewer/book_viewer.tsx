import {
  component$,
  useClientEffect$,
  useStylesScoped$,
} from "@builder.io/qwik";
import styles from "./book_viewer.css?inline";

export default component$((props: { input: string }) => {
  useStylesScoped$(styles);

  useClientEffect$(async ({ track }) => {
    track(() => props.input);
    const res = await fetch("https://book.zegs.me/default", {
      method: "POST",
      body: props.input,
    });

    const image_bytes = await res.arrayBuffer();
    const canvas = (
      document.getElementById("book-img") as HTMLCanvasElement
    ).getContext("2d");

    const img = new Image();
    img.onload = () => {
      canvas?.drawImage(img, 0, 0);
    };
    const blob = URL.createObjectURL(new Blob([image_bytes]));
    img.src = blob;
  });

  return (
    <div class="input-box">
      <canvas id="book-img" width={146} height={180}></canvas>
    </div>
  );
});
