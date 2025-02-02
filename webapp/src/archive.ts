export type Archive = (Solution | null)[]

export type Genotype = bigint[]

export interface Solution {
    seeds: Genotype
    measures: Float32Array
}

function clamp(number: number, min: number, max: number) {
    return Math.max(min, Math.min(number, max))
}

export const archive_rows = 20
export const archive_cols = 42

const limits = [
    { min: 70, max: 160 },
    { min: 1.3, max: 3.8 },
]

export function archive_bin(solution: Solution) {
    let m0_norm =
        (solution.measures[0] - limits[0].min) / (limits[0].max - limits[0].min)
    let m1_norm =
        (solution.measures[1] - limits[1].min) / (limits[1].max - limits[1].min)
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
