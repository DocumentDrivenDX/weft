; benchmark generated from python API
(set-info :status unknown)
(declare-fun b () Int)
(declare-fun masked () Bool)
(declare-fun a () Int)
(assert
 (let (($x13 (<= b 2)))
(let (($x12 (>= b 0)))
(let (($x11 (<= a 2)))
(let (($x9 (>= a 0)))
(let (($x14 (and $x9 $x11 $x12 $x13)))
(and $x14 masked (and (distinct (ite masked 3 a) (ite masked 3 b)) true))))))))
(check-sat)
