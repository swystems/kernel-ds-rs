import matplotlib.pyplot as plt
import matplotlib.ticker as mticker

MAX_TICKS = 200


def str2ts(s):
    token = s.split()
    return {'tag': token[0], 'ku': token[1], 'ticker': eval(token[2]), 'time': eval(token[3])}


def str2info(s: str):
    token = s.splitlines()
    return {'head': token[0].split()[0], 'timestamp': [str2ts(s) for s in token[2:]]}


def find(tag, ku, ticker, tss):
    return next(filter(lambda x: x['tag'] == tag and x['ku'] == ku and x['ticker'] == ticker, tss))


def outer_draw():
    counter = {}

    def inner_draw(x, data_groups, title, ylim=None):
        fig, ax = plt.subplots(1, 2, figsize=(12, 10), gridspec_kw={'width_ratios': [6, 1]})
        color = [plt.get_cmap('tab20c')(i) for i in range(7, -1, -1)]
        color_iter = iter(color)
        for data in data_groups:
            y = data['val']
            clr = next(color_iter)
            ax[0].scatter(x, y, color=clr, label='{}'.format(data['head']))

        ax[1].axis('off')
        ax[0].legend(bbox_to_anchor=(1, 1), loc='upper left')
        ax[0].set_title(title)
        ax[0].set_xlabel('ticker (n-th msg)')
        ax[0].set_ylabel('time / ns')
        ax[0].yaxis.set_major_formatter(mticker.FuncFormatter(lambda value, _: "{:,.0f}".format(value)))
        if ylim:
            ax[0].set_ylim(*ylim)
        # plt.show()
        fname = '{}_{}.png'.format(title, counter.setdefault(title, 0))
        plt.savefig(f'./pic/less_slots/{fname}')
        counter[title] += 1
    return inner_draw


draw = outer_draw()


# WriteStart -> WriteEnd
def write_copy(tasks, max_ticks=MAX_TICKS):
    x = list(range(max_ticks))
    data_groups = []
    for tsk in tasks:
        start = [find('WriteStart', 'kernel', i, tsk['timestamp']) for i in range(max_ticks)]
        end = [find('WriteEnd', 'kernel', i, tsk['timestamp']) for i in range(max_ticks)]
        y = [x[1]['time'] - x[0]['time'] for x in zip(start, end)]
        data_groups.append({'head': tsk['head'], 'val': y})

    draw(x, data_groups, 'write_copy')
    draw(x, data_groups, 'write_copy', (0, 500))
    draw(x, data_groups, 'write_copy', (10000, 60000))


def read_copy(tasks, max_ticks=MAX_TICKS):
    x = list(range(max_ticks))
    data_groups = []
    for tsk in tasks:
        ku = 'kernel' if 'fsrw' in tsk['head'] else 'user'
        start = [find('ReadStart', ku, i, tsk['timestamp']) for i in range(max_ticks)]
        end = [find('ReadEnd', ku, i, tsk['timestamp']) for i in range(max_ticks)]
        y = [x[1]['time'] - x[0]['time'] for x in zip(start, end)]
        data_groups.append({'head': tsk['head'], 'val': y})

    draw(x, data_groups, 'read_copy')
    draw(x, data_groups, 'read_copy', (0, 2500))
    draw(x, data_groups, 'read_copy', (200000, 800000))


def write_sync(tasks, max_ticks=MAX_TICKS):
    x = list(range(max_ticks))
    data_groups = []
    for tsk in tasks:
        start = [find('WriteSyncStart', 'kernel', i, tsk['timestamp']) for i in range(max_ticks)]
        end = [find('WriteSyncEnd', 'kernel', i, tsk['timestamp']) for i in range(max_ticks)]
        y = [x[1]['time'] - x[0]['time'] for x in zip(start, end)]
        data_groups.append({'head': tsk['head'], 'val': y})

    draw(x, data_groups, 'write_sync')
    draw(x, data_groups, 'write_sync', (0, 20000))
    draw(x, data_groups, 'write_sync', (0, 500))


def read_sync(tasks, max_ticks=MAX_TICKS):
    x = list(range(max_ticks))
    data_groups = []
    for tsk in tasks:
        ku = 'kernel' if 'fsrw' in tsk['head'] else 'user'
        start = [find('ReadSyncStart', ku, i, tsk['timestamp']) for i in range(max_ticks)]
        end = [find('ReadSyncEnd', ku, i, tsk['timestamp']) for i in range(max_ticks)]
        y = [x[1]['time'] - x[0]['time'] for x in zip(start, end)]
        data_groups.append({'head': tsk['head'], 'val': y})

    draw(x, data_groups, 'read_sync')
    draw(x, data_groups, 'read_sync', (0, 1000))
    # draw(x, data_groups, 'read_copy', (400000, 600000))


def read_total(tasks, max_ticks=MAX_TICKS):
    x = list(range(max_ticks))
    data_groups = []
    for tsk in tasks:
        start_tag = 'ReadStart' if 'fsrw' in tsk['head'] else 'ReadSyncStart'
        start = [find(start_tag, 'user', i, tsk['timestamp']) for i in range(max_ticks)]
        end = [find('ReadEnd', 'user', i, tsk['timestamp']) for i in range(max_ticks)]
        y = [x[1]['time'] - x[0]['time'] for x in zip(start, end)]
        data_groups.append({'head': tsk['head'], 'val': y})

    draw(x, data_groups, 'read_total')
    draw(x, data_groups, 'read_total', (0, 10000))
    draw(x, data_groups, 'read_total', (300000, 800000))


def main():
    with open('log') as f:
        log = f.read()
    tasks = [x.strip() for x in log.split('@')][1:]
    tasks = [str2info(s) for s in tasks]
    tasks = [{'head': t['head'], 'timestamp': [ts for ts in t['timestamp'] if ts['ticker'] < MAX_TICKS]} for t in tasks]
    # fsrw_tasks = [t for t in tasks if 'fsrw' in t['head']]
    # mmap_tasks = [t for t in tasks if 'mmap' in t['head']]
    write_copy(tasks)
    read_copy(tasks)
    write_sync(tasks)
    read_sync(tasks)
    read_total(tasks)


if __name__ == '__main__':
    main()

# read_sync_time = 0
# read_time = 0
# for i in range(80):
#     read_sync_start = find('ReadSyncStart', 'kernel', i, log_more_slot)
#     read_sync_end = find('ReadSyncStart', 'kernel', i, log_more_slot)
#     delt = read_sync_end['time'] - read_sync_start['time']
#     assert delt >= 0
#     read_sync_time += delt
#
# print(read_sync_time)
