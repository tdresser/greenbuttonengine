import { Signal, signal } from '@preact/signals';
import '@picocss/pico/css/pico.classless.css';
import { DateAggregatedCharts } from './charts/date_aggregated_charts.ts';
import './index.css';

import init, {
  get_timeseries,
  get_timeseries_chunked,
  ingest_xml,
  TimeSeries,
} from '../../lib/wasm/pkg/wasm';

const denormalized: Signal<TimeSeries[]> = signal([]);

const errorsEl = document.getElementById('errors')!;

// From https://gist.github.com/Lehoczky/241f046b05c54af65918676888fc783f.
function download(
  fileName: string,
  data: string | Uint8Array,
  mime = 'text/plain',
  bom?: string | Uint8Array,
) {
  const blobData = bom === undefined ? [data] : [bom, data];
  const blob = new Blob(blobData, { type: mime });
  const a = document.createElement('a');

  a.download = fileName;
  a.href = URL.createObjectURL(blob);
  a.click();
  // There's no rush to revoke this, but we should do it at some point.
  setTimeout(() => {
    URL.revokeObjectURL(a.href);
    a.remove();
  }, 1000);
}

(async function main() {
  // In theory, all the wasm stuff could live in a worker.
  // For simplicity of this demo, all this just runs on main.
  await init();

  document.getElementById('get_csv')!.addEventListener('click', () => {
    const timeseries = get_timeseries();
    download('timeseries.csv', timeseries.asCSV(), 'text/csv');
  });

  document.getElementById('get_influx')!.addEventListener('click', () => {
    const timeseries = get_timeseries();
    download('timeseries.txt', timeseries.asInfluxdb(), 'text/plain');
  });

  document.getElementById('get_parquet')!.addEventListener('click', () => {
    const timeseries = get_timeseries();
    download(
      'timeseries.parquet',
      timeseries.asParquet(),
      'application/octet-stream',
    );
  });

  function onFail(msg: string) {
    console.error(msg);
    errorsEl.innerHTML += `
    <p class="error">${msg}</p>
  `;
  }

  const getFiles = document.getElementById('getFiles')!;
  const dragHighlight = document.getElementById('draghighlight')!;
  let dragging = false;

  // From https://stackoverflow.com/a/46568146 .
  async function readFile(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      var fr = new FileReader();
      fr.onload = () => {
        if (typeof fr.result == 'string') {
          resolve(fr.result);
        } else {
          throw 'Non string file parse: ' + typeof fr.result;
        }
      };
      fr.onerror = reject;
      fr.readAsText(file);
    });
  }

  getFiles.addEventListener('click', async () => {
    errorsEl.innerHTML = '';

    const handles = await window.showOpenFilePicker({ multiple: true });
    for (const handle of handles) {
      try {
        const file = await handle.getFile();
        const contents = await readFile(file);
        ingest_xml(contents, file.name);
      } catch (e: any) {
        onFail(e.toString());
      }
    }

    denormalized.value = get_timeseries_chunked();
  });

  document.documentElement.addEventListener('dragover', (event: DragEvent) => {
    event.preventDefault();
    if (!dragging) {
      dragging = true;
      dragHighlight.classList.add('dragover');
    }
  });

  document.documentElement.addEventListener('dragleave', (event: DragEvent) => {
    // Dragleave events fire too often. Looking at the target is unreliable,
    // so we just look at the coordinates instead.
    if (
      0 < event.clientX &&
      event.clientX < window.innerWidth &&
      0 < event.clientY &&
      event.clientX < window.innerHeight
    ) {
      return;
    }
    dragging = false;
    dragHighlight.classList.remove('dragover');
  });

  document.documentElement.addEventListener(
    'drop',
    async (event: DragEvent) => {
      event.preventDefault();
      dragging = false;
      dragHighlight.classList.remove('dragover');
      if (!event.dataTransfer) {
        return;
      }
      for (const item of event.dataTransfer.items) {
        if (item.kind === 'file') {
          const file = item.getAsFile();
          if (!file) {
            continue;
          }
          const xml = await readFile(file);
          ingest_xml(xml, file.name);
        }
      }
      denormalized.value = get_timeseries_chunked();
    },
  );

  new DateAggregatedCharts(
    document.getElementById('charts')!,
    document.getElementById('chartZoomboxes')!,
    denormalized,
  );
})();
