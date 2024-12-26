import React, { useEffect, useState } from 'react';
import prettier from 'prettier/standalone';
import htmlParser from 'prettier/plugins/html';
import { Light as SyntaxHighlighter } from 'react-syntax-highlighter';
import xml from 'react-syntax-highlighter/dist/esm/languages/hljs/xml';
import theme from 'react-syntax-highlighter/dist/esm/styles/hljs/tomorrow-night-blue';
import Header from './Header';

SyntaxHighlighter.registerLanguage('html', xml);

declare global {
  interface Window {
    convert?: (adoc: string, timestamp: number) => string;
  }
}

type ParsedResult =
  | { success: true; html: string }
  | { success: false; errors: string[] };

const App: React.FC = () => {
  const [adoc, setAdoc] = useState('Hello, *AsciiDork!*');
  const [html, setHtml] = useState(DEFAULT_HTML);
  const [error, setError] = useState(false);

  useEffect(() => {
    async function inner() {
      if (!window.convert) return;
      console.time(`converted in`);
      const result = window.convert(adoc, Math.floor(Date.now() / 1000));
      console.timeEnd(`converted in`);
      const parsed: ParsedResult = JSON.parse(result);
      if (parsed.success) {
        try {
          const pretty = await prettier.format(parsed.html, {
            parser: 'html',
            plugins: [htmlParser],
            printWidth: 80,
          });
          setError(false);
          setHtml(pretty);
        } catch (e) {
          console.error('Failed to format HTML');
          console.log(parsed.html);
          setError(true);
          setHtml((e as Error).message);
        }
      } else {
        setError(true);
        setHtml(parsed.errors.join('\n\n'));
      }
    }
    inner();
  }, [adoc]);

  return (
    <div className="flex flex-col min-h-screen antialiased">
      <Header />
      <div className="flex grow font-mono">
        <textarea
          spellCheck={false}
          className="w-2/5 bg-gray-100 p-2 font-mono font-semibold text-blue-900 focus:outline-none"
          value={adoc}
          onChange={(e) => setAdoc(e.target.value)}
        />
        {!error ? (
          <SyntaxHighlighter wrapLines class="w-3/5" language="html" style={theme}>
            {html}
          </SyntaxHighlighter>
        ) : (
          <pre className="w-3/5 p-2 overflow-auto whitespace-wrap bg-black text-red-500 font-bold text-sm">
            {html}
          </pre>
        )}
      </div>
    </div>
  );
};

export default App;

const DEFAULT_HTML = `<div class="paragraph">
  <p>Hello, <strong>Asciidork!</strong></p>
</div>`;
