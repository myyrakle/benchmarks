import time
import pandas 
import resource

class Timer:
    def __init__(self):
        self.start = time.time()
        
    def elapsed_as_milliseconds(self):
        return (time.time() - self.start) * 1000

def do_something():
    dataframe = pandas.read_csv('measurements.txt', sep=';', names=['name', 'value'])
    filtered = dataframe[dataframe['value'] < 5000]
    result = filtered.groupby('name').agg({
        'value': ['min', 'max', 'mean', 'sum', 'count']
    })
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
