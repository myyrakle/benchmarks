import time
import polars 
import resource

class Timer:
    def __init__(self):
        self.start = time.time()
        
    def elapsed_as_milliseconds(self):
        return (time.time() - self.start) * 1000

def do_something():
    dataframe = polars.scan_csv('measurements.txt', separator=';', new_columns=['name', 'value'])
    filtered = dataframe.filter(polars.col('value') < 5000)
    result = filtered.groupby('name').agg(
        polars.col('value').min().alias('min'),
        polars.col('value').max().alias('max'),
        polars.col('value').mean().alias('mean'),
        polars.col('value').sum().alias('sum'),
        polars.col('value').count().alias('count')
    ).collect()
    return result

def main():
    timer = Timer()
    result = do_something()
    elapsed_time = timer.elapsed_as_milliseconds()
    print(result)
    memory_usage_kb= resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    print(f"Elapsed time: {elapsed_time} ms")
    print(f"Memory usage: {memory_usage_kb} KB")

if __name__ == "__main__":
    main()
