import React, { useEffect, useState } from 'react';
import prettier from 'prettier/standalone';
import htmlParser from 'prettier/plugins/html';
import cx from 'classnames';

declare global {
  interface Window {
    convert?: (adoc: string) => string;
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
      const result = window.convert(adoc);
      console.timeEnd(`converted in`);
      const parsed: ParsedResult = JSON.parse(result);
      if (parsed.success) {
        try {
          const pretty = await prettier.format(parsed.html, {
            parser: 'html',
            plugins: [htmlParser],
            printWidth: 60,
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
    <div className="flex min-h-screen font-mono">
      <textarea
        spellCheck={false}
        className="w-2/5 bg-gray-50 p-2 focus:outline-none"
        value={adoc}
        onChange={(e) => setAdoc(e.target.value)}
      />
      <pre
        className={cx(
          `w-3/5 p-2 overflow-scroll whitespace-wrap`,
          error && `text-red-700 text-xs`,
        )}
      >
        {html}
      </pre>
    </div>
  );
};

export default App;

const DEFAULT_HTML = `<div class="paragraph">
  <p>Hello, <strong>Asciidork!</strong></p>
</div>`;
