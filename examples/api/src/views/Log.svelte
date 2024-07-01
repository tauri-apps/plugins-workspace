<script>
    function test() {
        // This file is used to check the output of exposed console methods.
        // All top-level exposed functions of the official console API are included.
        // https://console.spec.whatwg.org/#console-namespace

        // Includes all tests in https://github.com/ioannad/console/blob/main/NOTES.md

        // This test should be run first because it contains a `console.clear` which may
        // influence subsequent tests.
        function test_console_methods_basic() {
            console.log(
                "------- Testing console methods for basic functionality -------",
            );

            // assert
            console.log(' --- console.assert(true, "should not be logged");');
            console.assert(true, "should not be logged");

            console.log(
                ' --- console.assert(false, "Failed assertion logged.", "one", "two", 3);',
            );
            console.assert(false, "Failed assertion logged.", "one", "two", 3);

            console.log(
                " --- console.assert(console.assert(true) == undefined);",
            );
            console.assert(console.assert(true) == undefined);

            console.log(' --- console.log("does this appear?");');
            console.log("does this appear?"); //  it does when streams are being redirected to files.

            console.log(' --- console.group("foo");');
            console.group("foo");

            console.log(' --- console.log("should not appear");');
            console.log("should not appear"); // still does when streams are being redirected to files.

            console.log(" --- console.clear();");
            console.clear();

            console.log(' --- console.debug("debug");');
            console.debug("debug");

            console.log(' --- console.error("error");');
            console.error("error");

            console.log(' --- console.info("info");');
            console.info("info");

            console.log(' --- console.log("log");');
            console.log("log");

            console.log(' --- console.warn("warn");');
            console.warn("warn");

            // Due to some weird formatting difference between native node.js and deno run
            // outputs, suspecting that console.clear somehow left open the group that was
            // created before the clearing of the console.

            console.log(" --- console.groupEnd();");
            console.groupEnd();

            function tracedfoo() {
                function tracedbar() {
                    console.trace();
                }
                tracedbar();
            }

            console.log(" --- tracedfoo();");
            tracedfoo();

            console.log(' --- console.dir(["dir", 2]);');
            console.dir(["dir", 2]);

            // ~data~ is any object or DOM element - Nodejs documentation says that it treats ~dirxml~ as ~log~.
            console.log(' --- console.dirxml([1,2], "default", null);');
            console.dirxml([1, 2], "default", null);

            // Grouping
            console.log(' --- console.group("bar");');
            console.group("bar");

            console.log(' --- console.groupCollapsed("baz");');
            console.groupCollapsed("baz");

            console.log(" --- console.groupEnd();");
            console.groupEnd();

            console.log(" --- console.groupEnd();");
            console.groupEnd();

            console.log(function () {});
            console.log(async function anAsyncFunction(a, b, c) {});
            console.log(function* aGeneratorFunction() {});
            console.log(async function* anAsyncGeneratorFunction() {});

            console.log(new Int8Array([1, 2, 3, 4]));
            console.log(new Uint8Array([1, 2, 3, 4]));
            console.log(new Uint8ClampedArray([1, 2, 3, 4]));
            console.log(new Int16Array([1, 2, 3, 4]));
            console.log(new Uint16Array([1, 2, 3, 4]));
            console.log(new Int32Array([1, 2, 3, 4]));
            console.log(new Uint32Array([1, 2, 3, 4]));
            console.log(new Float32Array([1, 2, 3, 4]));
            console.log(new Float64Array([1, 2, 3, 4]));
            console.log(new BigInt64Array([1n, 2n, 3n, 4n]));
            console.log(new BigUint64Array([1n, 2n, 3n, 4n]));

            console.log(
                new Map([
                    ["a", 1],
                    ["b", 2],
                ]),
            );
            console.log(new Set([1, 2, 3, 4]));
        }

        function test_formatter() {
            // Initially taken from https://github.com/ioannad/console/blob/main/NOTES.md

            console.log("------- Testing formatter -------");

            // Float formatting of integers
            console.log(' --- console.log("%f", 23);');
            console.log("%f", 23);

            console.log(
                " --- console.log('bjoern and robert are born on the %fst dec', 1.234);",
            );
            console.log("bjoern and robert are born on the %fst dec", 1.234);

            console.log(' --- console.log("%f", null);');
            console.log("%f", null);

            // Integer formatting of null
            console.log(' --- console.log("%d", null);');
            console.log("%d", null);

            // Integer formatting of a string
            console.log(
                " --- console.log('bjoern and robert are born on the %dst dec', \"foo\");",
            );
            console.log("bjoern and robert are born on the %dst dec", "foo");

            // not enough arguments to interpolate all placeholders
            console.log(' --- console.log("%s %snewword", "duck");');
            console.log("%s %snewword", "duck");

            // console.assert - string formatter in assert
            console.log(
                ' --- console.assert(false, "robert keeps %s on his balcony", "plaices");',
            );
            console.assert(false, "robert keeps %s on his balcony", "plaices");

            // console.assert - string formatter of an object
            console.log(
                ' --- console.assert(false, "robert keeps %s on his balcony", {foo: "bar"});',
            );
            console.assert(false, "robert keeps %s on his balcony", {
                foo: "bar",
            });
        }

        function test_console_table() {
            console.log("------- Testing console.table -------");

            // console.table - printing of strings
            console.log(
                ' --- console.table("the plaice living on the balcony");',
            );
            console.table("the plaice living on the balcony");

            // console.table - Sets
            console.log(
                ' --- console.table(new Set([{name: "terin", owner: false}, {name: "robert", owner: false}, {name: "domenic", owner: true}]));',
            );
            console.table(
                new Set([
                    { name: "terin", owner: false },
                    { name: "robert", owner: false },
                    { name: "domenic", owner: true },
                ]),
            );

            // console.table - Multiple Arguments
            console.log(
                " --- console.table([[1, 2, 3, 4], [5, 6, 7, 8]], 2, 3);",
            );
            //    console.log('Fails in Deno repl 1.34.2'); // ------------------------------ FAIL
            console.table(
                [
                    [1, 2, 3, 4],
                    [5, 6, 7, 8],
                ],
                2,
                3,
            );
        }

        function test_console_count() {
            console.log("------- Testing console.count -------");

            // Basic functionality

            console.log(" --- console.count();");
            console.count();

            console.log(' --- console.countReset("default");');
            console.countReset("default");

            console.log(' --- console.count("default");');
            console.count("default");

            // console.count - counters and label repetition

            console.log(" --- console.count('foo');");
            console.count("foo");
            console.log(" --- console.count('foo');");
            console.count("foo");

            // console.count - objects / arrays

            console.log(" --- console.count({});");
            console.count({});

            console.log(" --- console.count([]);");
            console.count([]);

            // console.count - no arguments / empty strings / null / undefined

            console.log(" --- console.count();");
            console.count();

            console.log(' --- console.count("");');
            console.count("");

            console.log(" --- console.count(null);");
            console.count(null);

            console.log(" --- console.count(undefined);");
            console.count(undefined);
        }

        function test_console_timing() {
            console.log("------- Testing console.time -------");

            console.log(" --- console.time();");
            console.time();

            console.log(" --- console.time(undefined);");
            console.time(undefined);

            console.log(" --- console.time(null);");
            console.time(null);

            console.log("------- Testing console.timeLog -------");

            console.log(" --- console.timeLog();");
            console.timeLog();

            console.log(" --- console.timeLog(undefined);");
            console.timeLog(undefined);

            console.log(" --- console.timeLog(null);");
            console.timeLog(null);

            console.log("------- Testing console.timeEnd -------");

            console.log(" --- console.timeEnd();");
            console.timeEnd();

            console.log(" --- console.timeEnd(null);");
            console.timeEnd(null);

            console.log(" --- console.timeEnd(undefined);");
            console.timeEnd(undefined);

            console.log(" --- Finish testing console.time");
        }

        // This should go first because it contains a "clear" that may mees up the output in some environments.
        test_console_methods_basic();

        test_formatter();
        test_console_count();
        test_console_timing();

        // This should go last because its last command fails in Deno 1.34.2 which stops execution.
        test_console_table(); // **TODO** consider moving this elsewhere and/or running it separately from the other tests

        console.log("---------------------------TEST END.");
    }
</script>

<button class="btn" on:click={test}> Test </button>
