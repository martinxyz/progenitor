export type Archive = (Solution | null)[]

export interface Solution {
    seed: bigint
    measures: Float32Array
}

function clamp(number: number, min: number, max: number) {
    return Math.max(min, Math.min(number, max))
}

export const archive_rows = 20
export const archive_cols = 42

const measures_max = [150, 110]

export function archive_bin(solution: Solution) {
    let m0_norm = solution.measures[0] / measures_max[0]
    let m1_norm = solution.measures[1] / measures_max[1]
    let col = clamp(Math.floor(m0_norm * archive_cols), 0, archive_cols)
    let row = clamp(Math.floor(m1_norm * archive_rows), 0, archive_rows)
    return col * archive_rows + row
}

export function extend_archive(a: Archive, a2: Archive): void {
    a2.forEach((solution, idx) => {
        if (!a[idx]) {
            a[idx] = solution
        }
    })
}
