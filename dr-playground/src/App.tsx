import React, { useEffect, useState } from 'react';
import prettier from 'prettier/standalone';
import htmlParser from 'prettier/plugins/html';

declare global {
  interface Window {
    convert?: (adoc: string) => string;
  }
}

const App: React.FC = () => {
  const [adoc, setAdoc] = useState('Hello, *AsciiDork!*');
  const [html, setHtml] = useState(DEFAULT_HTML);

  useEffect(() => {
    async function inner() {
      if (!window.convert) return;
      console.time(`converted in`);
      const ugly = window.convert(adoc);
      console.timeEnd(`converted in`);
      const pretty = await prettier.format(ugly, {
        parser: 'html',
        plugins: [htmlParser],
        printWidth: 50,
      });
      setHtml(pretty);
    }
    inner();
  }, [adoc]);

  return (
    <div className="flex min-h-screen font-mono">
      <textarea
        spellCheck={false}
        className="w-1/2 bg-gray-50 p-2 focus:outline-none"
        value={adoc}
        onChange={(e) => setAdoc(e.target.value)}
      />
      <pre className="w-1/2 p-2 overflow-scroll whitespace-wrap">{html}</pre>
    </div>
  );
};

export default App;

const DEFAULT_HTML = `<div class="paragraph">
  <p>Hello, <strong>Asciidork!</strong></p>
</div>`;
