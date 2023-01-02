use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Div, Mul, Sub};

/// 自定义类型V的D维的矩形
///
/// ```rust
/// use MerkleRTree::shape::Rect;
/// let mut r = Rect::new([1,2], [3,4]);
/// println!("{:?}", r)
/// ```
#[derive(Debug, Clone)]
pub struct Rect<V, const D: usize>
    where
        V: Default + Debug + Copy,
{
    pub _max: [V; D],
    pub _min: [V; D],
}

fn min<V: PartialOrd>(a: V, b: V) -> V {
    if a < b {
        a
    } else {
        b
    }
}

fn max<V: PartialOrd>(a: V, b: V) -> V {
    if a > b {
        a
    } else {
        b
    }
}

impl<V, const D: usize> Rect<V, D> 
where
    V: Default + Debug + Copy,
{
    pub fn new(min: [V; D], max: [V; D]) -> Self {
        Self {
            _min: min,
            _max: max,
        }
    }

    pub fn new_point(point: [V; D]) -> Self {
        Self {
            _min: point.clone(),
            _max: point,
        }
    }
}

impl<V, const D: usize> Display for Rect<V, D>
where
    V: Default + Debug + Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{:?}, {:?}}}", self._min, self._max)
    }
}

impl<V, const D: usize> Rect<V, D>
where
    V: Default + Debug + Copy,
    V: PartialOrd + Sub<Output=V> + Add<Output=V> + Mul<Output=V> + Div<Output=V>,
{

    pub fn expand(&mut self, rect: &Rect<V, D>) {
        for i in 0..D {
            if rect._min[i] < self._min[i] {
                self._min[i] = rect._min[i];
            }
            if rect._max[i] > self._max[i] {
                self._max[i] = rect._max[i];
            }
        }
    }

    pub fn largest_axis(&self) -> usize {
        if D == 0 {
            return 0;
        }
        let mut axis = 0_usize;
        let mut size = self._max[0] - self._min[0];
        for i in 1..D {
            let asize = self._max[i] - self._min[i];
            if asize > size {
                axis = 1;
                size = asize;
            }
        }
        axis
    }

    /// 判断是否包含D维矩形rect，包含边界
    pub fn contains(&self, rect: &Rect<V, D>) -> bool {
        if D == 0 {
            return false;
        }
        for i in 0..D {
            if rect._min[i] < self._min[i] ||
                rect._max[i] > self._max[i] {
                return false;
            }
        }
        true
    }

    /// 判断两个D维矩形是否相交
    pub fn intersects(&self, rect: &Rect<V, D>) -> bool {
        if D == 0 {
            return false;
        }
        for i in 0..D {
            if rect._max[i] < self._min[i] ||
                rect._min[i] > self._max[i] {
                return false;
            }
        }
        true
    }

    /// 判断D维矩形rect是否与边界相交，当rect表现为点时，表示点是否在边界上
    pub fn on_edge(&self, rect: &Rect<V, D>) -> bool {
        for i in 0..D {
            if !(rect._min[i] > self._min[i] && rect._max[i] < self._max[i]) {
                return true;
            }
        }
        false
    }

    /// 计算D维矩形测度。如果维度为0，则返回V的默认值。
    /// 二维的测度为面积，三维的测度为体积
    pub fn area(&self) -> V {
        if D == 0 {
            return V::default();
        }
        let mut area = self._max[0] - self._min[0];
        for i in 1..D {
            area = area * (self._max[i] - self._min[i]);
        }
        area
    }

    /// 计算两个D维矩形合并以后的边积，如果维度为0，则返回V的默认值
    pub fn unioned_area(&self, rect: &Rect<V, D>) -> V {
        if D == 0 {
            return V::default();
        }
        let mut area = max(self._max[0], rect._max[0]) - min(self._min[0], rect._min[0]);
        for i in 1..D {
            area = area * (max(self._max[i], rect._max[i]) - min(self._min[i], rect._min[i]));
        }
        area
    }

    /// 计算两个D维矩形间的距离，当两个矩形在任意维度上都没有交集时，表现为最接近的两个顶点的距离,
    /// 距离为欧氏距离的平方
    pub fn rect_dist(&self, rect: &Rect<V, D>) -> V {
        let zero = V::default();
        if D == 0 {
            return zero;
        }
        let mut dist = zero;
        for i in 0..D {
            let d = max(self._min[i], rect._min[i]) - min(self._max[i], rect._max[i]);
            if d > zero {
                dist = dist + (d * d);
            }
        }
        dist
    }

    /// 计算两个矩形的重叠测度，不重叠时返回V的默认值
    pub fn overlap_area(&self, rect: &Rect<V, D>) -> V {
        let zero = V::default();
        if D == 0 {
            return zero;
        }
        let mut area = min(self._max[0], rect._max[0]) - max(self._min[0], rect._min[0]);
        if !(area > zero) {
            return zero;
        }
        for i in 1..D {
            let d = min(self._max[i], rect._max[i]) - max(self._min[i], rect._min[i]);
            if !(d > zero) {
                return zero;
            }
            area = area * d;
        }
        area
    }

    /// 输出信息
    pub fn display(&self) -> String {
        format!("{{{:?}, {:?}}}", self._min, self._max)
    }
}

impl<V, const D: usize> Default for Rect<V, D> 
where
    V: Default + Debug + Copy,
{
    fn default() -> Self {
        Self {
            _max: [Default::default(); D], 
            _min: [Default::default(); D] 
        }
    }
}