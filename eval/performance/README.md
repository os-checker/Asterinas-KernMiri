# Performance evaluation

## Running benchmarks

To execute benchmarks and get normalized results (Linux/Asterinas ratios for latency or Asterinas/Linux for bandwidth), including geometric means:

```shell
export BENCHMARK={bench_name} # lmbench|nginx|redis|sqlite
cd $BENCHMARK && ./run_all.sh
```

Results will be saved in `./result/{bench_name}/` containing:

- Raw outputs: Multiple `result_*.json` files
- Normalized summaries: Single `Ratio-summary.json` file

## Note on Redis and SQLite Results

The test scripts do not extract all Redis and SQLite benchmark data into `.json` files. Complete results are preserved in the `./results/{bench_name}` directory. Below are instructions for obtaining their full results:

**Redis:**

1. Find two CSV files (`all-*.csv`) in `./result/redis/` with this header format:

```text
"test","rps","avg_latency_ms","min_latency_ms","p50_latency_ms","p95_latency_ms","p99_latency_ms","max_latency_ms"
```

2. Use `stat` to identify results: Asterinas (earlier timestamp) and Linux (later timestamp).

**SQLite:**

Check `./result/sqlite/` for:

- `aster_output.txt` (Asterinas)
- `linux_output.txt` (Linux)

## Result Analysis

To generate summarized results (CSV tables and PNG plots), follow these steps:

1. **First**, execute the test script. By default, it will run 10 iterations of each benchmark:

    ```shell
        cd utils && ./multiple_run.sh
    ```

2. Generate Analysis Reports:
Run the following scripts to process the raw data and produce outputs:

    |Test|Command|Output Files|
    |---|---|---|
    |LMbench|cd utils/analysis && python lmbench.py|lmbench_results.csv|
    |Nginx|cd utils/analysis && python nginx.py|nginx_results.csv, nginx_results_bar.png|
    |SQLite|cd utils/analysis && python sqlite.py|sqlite_results.csv, sqlite_results_bar.png|
    |Redis|cd utils/analysis && python redis.py|redis_results.csv, redis_results_bar.png|

3. All results will be saved in `utils/analysis/`.
